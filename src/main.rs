use std::collections::HashSet;
use std::{collections::HashMap, fs::File, io::BufRead};
use std::{io::Read, usize};
use wasmparser::Operator;

#[derive(Debug)]
struct MessageError(String);
impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for MessageError {}

fn find_alloca(op_reader: &mut wasmparser::OperatorsReader<'_>, stack_ptr_global_idx: u32) -> bool {
    let mut global_set_count = 0;
    while let Ok(op) = op_reader.read() {
        if let Operator::GlobalSet { global_index } = op {
            if global_index == stack_ptr_global_idx {
                global_set_count += 1
            }
        }
    }
    global_set_count > 1
}

fn collect_instr_until_set_sp<'body>(
    body: &'body wasmparser::FunctionBody,
    stack_ptr_global_idx: u32,
    instrs: &mut Vec<Operator<'body>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut op_reader = body.get_operators_reader()?;
    while let Ok(op) = op_reader.read() {
        if let Operator::GlobalSet { global_index } = op {
            instrs.push(Operator::GlobalSet { global_index });
            if global_index == stack_ptr_global_idx {
                if find_alloca(&mut op_reader, stack_ptr_global_idx) {
                    return Err(Box::new(MessageError(
                        "more than two set operation for stack pointer found".to_string(),
                    )));
                }
                return Ok(());
            }
        }
        instrs.push(op);
    }
    Err(Box::new(MessageError(
        "no set operation for stack pointer found".to_string(),
    )))
}

fn estimate_stack_alloc_size(
    body: wasmparser::FunctionBody,
    stack_ptr_global_idx: u32,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut instrs = vec![];
    collect_instr_until_set_sp(&body, stack_ptr_global_idx, &mut instrs)?;

    #[allow(unused)]
    #[derive(Clone)]
    enum Expr {
        UnknownGlobal { index: u32 },
        UnknownLocal { index: u32 },
        Immediate { value: usize },
        Computed { base: Box<Expr>, minus: usize },
    }
    impl Expr {
        fn sub(&self, rhs: &Expr) -> Expr {
            match (self, rhs) {
                (Self::UnknownGlobal { .. }, Self::Immediate { value }) => {
                    Expr::Computed {
                        base: Box::new(self.clone()),
                        minus: *value,
                    }
                }
                _ => todo!("unsupported expr operation"),
            }
        }
    }
    let mut globals = HashMap::<u32, Expr>::new();
    let mut locals = HashMap::<u32, Expr>::new();
    let mut stack = Vec::<Expr>::new();

    for op in instrs {
        match op {
            Operator::GlobalGet { global_index } => {
                if let Some(found) = globals.get(&global_index) {
                    stack.push(found.clone());
                } else {
                    stack.push(Expr::UnknownGlobal {
                        index: global_index,
                    });
                }
            }
            Operator::GlobalSet { global_index } => {
                globals.insert(global_index, stack.pop().unwrap());
            }
            Operator::LocalGet { local_index } => {
                if let Some(found) = locals.get(&local_index) {
                    stack.push(found.clone());
                } else {
                    // TODO: Non-arg locals are known to be initialized as zero
                    stack.push(Expr::UnknownLocal { index: 0 });
                }
            }
            Operator::LocalTee { local_index } => {
                let value = stack.last().unwrap().clone();
                locals.insert(local_index, value);
            }
            Operator::LocalSet { local_index } => {
                locals.insert(local_index, stack.pop().unwrap());
            }
            Operator::I32Const { value } => {
                stack.push(Expr::Immediate { value: value as _ });
            }
            Operator::I32Sub => {
                let rhs = stack.pop().unwrap();
                let lhs = stack.pop().unwrap();
                stack.push(lhs.sub(&rhs));
            }
            _ => {
                return Err(Box::new(MessageError(format!(
                    "Unsupported prologue instruction: {:?}",
                    op
                ))))
            }
        }
    }
    if let Some(Expr::Computed { base, minus }) = globals.get(&stack_ptr_global_idx) {
        if let Expr::UnknownGlobal { index: base_index } = base.as_ref() {
            if *base_index == stack_ptr_global_idx {
                return Ok(*minus);
            }
        }
    }

    Err(Box::new(MessageError("stack pointer is not changed?".to_string())))
}

fn collect_func_names(
    mut reader: wasmparser::NameSectionReader,
    func_idx_by_name: &mut HashMap<String, usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    while !reader.eof() {
        let name = match reader.read() {
            Ok(name) => name,
            Err(_) => return Ok(()),
        };
        match name {
            wasmparser::Name::Module(_) => continue,
            wasmparser::Name::Function(n) => {
                let mut map = n.get_map()?;
                for _ in 0..map.get_count() {
                    let naming = map.read()?;
                    func_idx_by_name.insert(naming.name.to_owned(), naming.index as usize);
                }
            }
            wasmparser::Name::Local(_) => continue,
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: wasm-stack-consumer [.wasm file] [stacktrace]");
        std::process::exit(1);
    }
    let mut binary_file = File::open(&args[1])?;
    let stacktrace_path = File::open(&args[2])?;
    let stack_ptr_global_idx = if args.len() >= 4 {
        args[3].parse::<u32>().expect("invalid sp global index")
    } else {
        0
    };
    let mut size_by_idx = Vec::new();
    let mut func_names = HashMap::new();
    let mut func_idx_base = 0;

    let mut buffer = Vec::new();
    binary_file.read_to_end(&mut buffer).unwrap();
    let parser = wasmparser::Parser::new(0);

    for payload in parser.parse_all(&buffer) {
        use wasmparser::Payload;
        match payload? {
            Payload::ImportSection(mut section) => {
                use wasmparser::SectionReader;
                while !section.eof() {
                    let entry = section.read()?;
                    match entry.ty {
                        wasmparser::ImportSectionEntryType::Function(_) => {
                            func_idx_base += 1;
                        }
                        _ => continue,
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                size_by_idx.push(estimate_stack_alloc_size(body, stack_ptr_global_idx));
            }
            Payload::CustomSection {
                name,
                data,
                data_offset,
                range: _,
            } => if let "name" = name {
                let section = wasmparser::NameSectionReader::new(data, data_offset)?;
                collect_func_names(section, &mut func_names)?;
            },
            _ => continue,
        }
    }

    let reader = std::io::BufReader::new(stacktrace_path);
    let mut total = 0;
    let mut known_failed = HashSet::<String>::new();
    for line in reader.lines() {
        let line = line?;
        let idx = match func_names.get(&line) {
            Some(idx) => idx,
            None => {
                if known_failed.insert(line.clone()) {
                    eprintln!("{} not found", line);
                }
                continue;
            }
        };
        let size = match size_by_idx.get(*idx - func_idx_base) {
            Some(Ok(size)) => size,
            Some(Err(e)) => {
                if known_failed.insert(line.clone()) {
                    eprintln!("can't estimate stack size for '{}' ({:})", line, e);
                }
                continue;
            }
            None => {
                if known_failed.insert(line.clone()) {
                    eprintln!(
                        "invalid wasm file: name section contains non-existing func-idx for {}",
                        line
                    );
                }
                continue;
            }
        };
        total += size;
        println!("func[{}] size = {} {}", *idx, size, line);
    }

    println!("Total size: {}", total);
    Ok(())
}
