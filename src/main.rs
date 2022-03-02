use std::{collections::HashMap, fs::File, io::BufRead};
use std::{io::Read, usize};

#[derive(Debug)]
struct MessageError(String);
impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for MessageError {}

fn estimate_stack_alloc_size(
    body: wasmparser::FunctionBody,
    stack_ptr_global_idx: u32,
) -> Result<usize, Box<dyn std::error::Error>> {
    use wasmparser::Operator;
    let mut op_reader = body.get_operators_reader()?;
    while let Ok(op) = op_reader.read() {
        match op {
            Operator::GlobalGet { global_index } if global_index == stack_ptr_global_idx => {}
            _ => continue,
        };
        let value = match op_reader.read()? {
            Operator::I32Const { value } => {
                match op_reader.read()? {
                    Operator::I32Sub => {}
                    Operator::LocalTee { .. } => {
                        if let Operator::I32Sub = op_reader.read()? {
                        } else {
                            continue;
                        }
                    }
                    _ => {
                        continue;
                    }
                }
                if let Operator::LocalTee { .. } = op_reader.read()? {
                } else {
                    continue;
                }
                if let Operator::GlobalSet { global_index } = op_reader.read()? {
                    if global_index == stack_ptr_global_idx {
                        value
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        return Ok(value as usize);
    }
    Err(Box::new(MessageError("prologue not found".to_string())))
}

fn collect_func_names(
    mut reader: wasmparser::NameSectionReader,
    func_names: &mut HashMap<String, usize>,
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
                    func_names.insert(naming.name.to_owned(), naming.index as usize);
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
            } => match name {
                "name" => {
                    let section = wasmparser::NameSectionReader::new(data, data_offset)?;
                    collect_func_names(section, &mut func_names)?;
                }
                _ => (),
            },
            _ => continue,
        }
    }

    let reader = std::io::BufReader::new(stacktrace_path);
    let mut total = 0;
    for line in reader.lines() {
        let line = line?;
        let idx = match func_names.get(&line) {
            Some(idx) => idx,
            None => {
                eprintln!("{} not found", line);
                continue;
            }
        };
        let size = match size_by_idx.get(*idx - func_idx_base) {
            Some(Ok(size)) => size,
            Some(Err(e)) => {
                eprintln!("couldn't estimate stack size {} ({:}", line, e);
                continue;
            }
            None => {
                eprintln!(
                    "invalid wasm file: name section contains non-existing func-idx for {}",
                    line
                );
                continue;
            }
        };
        total += size;
        println!("func[{}] size = {} {}", *idx, size, line);
    }

    println!("Total size: {}", total);
    Ok(())
}
