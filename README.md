# wasm-stack-consumer

A simple binary analyzer for stack allocation size in WebAssembly based on LLVM code generation.

## Installation

With Rust's package manager cargo, you can install via:

```console
$ cargo install --git https://github.com/kateinoigakukun/wasm-stack-consumer
$ wasm-stack-consumer --help
```

## Usage

### 1. Prepare list of functions to analyze

First, you need to prepare a list of function names you want to know the stack alloc size.
This list is usually taken from the stack trace at the crash point. Some standalone WebAssembly runtime like `wasmtime` and browsers provide this information. You may need to manually format the provided raw call stack into a list of function names by trimming unnecessary information like code address location and prefix '$' added to every wasm function name.

```
$ cat callstack.log
_swift_stdlib_fwrite_stdout
$sSS8withUTF8yxxSRys5UInt8VGKXEKlFSi_Tg507$sSRys5c67VGSis5Error_pIgydzo_ACSisAD_pIegyrzo_TR030$ss7_StdoutV5writeyySSFSiE18A7VGXEfU_Tf3nnpf_nTf1cn_n
$ss6_print_9separator10terminator2toySayypG_S2Sxzts16TextOutputStreamRzlFs7_StdoutV_Tg5
$ss5print_9separator10terminatoryypd_S2StFTm
$ss5print_9separator10terminatoryypd_S2StF
main
```

<details>
<summary>Original stack trace from Google Chrome</summary>

It contains non-WebAssembly function names like `wasiObject.wasiImport.<computed>`, and also some WebAssembly function names like `$_swift_stdlib_fwrite_stdout`.
WebAssembly function names are prefixed with `$` to distinguish them from non-WebAssembly function names.

And each frame line contains the code address location `(<file>:<line>)` at the end.

```
wasmFs.fs.writeSync (dev.js:8982)
(anonymous) (dev.js:2485)
(anonymous) (dev.js:2483)
(anonymous) (dev.js:2174)
wasiObject.wasiImport.<computed> (dev.js:9005)
$__wasi_fd_write (01c5d406:0x40fff6)
$writev (01c5d406:0x410760)
$__stdio_write (01c5d406:0x4107ea)
$__stdout_write (01c5d406:0x414f29)
$fwrite (01c5d406:0x4105a9)
$_swift_stdlib_fwrite_stdout (01c5d406:0x3c743a)
$$sSS8withUTF8yxxSRys5UInt8VGKXEKlFSi_Tg507$sSRys5c67VGSis5Error_pIgydzo_ACSisAD_pIegyrzo_TR030$ss7_StdoutV5writeyySSFSiE18A7VGXEfU_Tf3nnpf_nTf1cn_n (01c5d406:0xa3aee)
$$ss6_print_9separator10terminator2toySayypG_S2Sxzts16TextOutputStreamRzlFs7_StdoutV_Tg5 (01c5d406:0x1abcec)
$$ss5print_9separator10terminatoryypd_S2StFTm (01c5d406:0x1abdf9)
$$ss5print_9separator10terminatoryypd_S2StF (01c5d406:0x1a9f82)
$main (main.swift:1)
run (dev.js:8954)
```

</details>

### 2. Analyze stack allocations


Then, this analyzer can tell you the size of stack allocation for each function. You need to provide the `.wasm` file with debug info (especially `name` section) and the list of functions you want to analyze.

Some of functions in the given list may be missing due to the limitation of stack size estimation.
Diagnostics for such missing functions are printed to stderr.

```
$ cargo run -- main.wasm callstack.log 2> /dev/null
func[6757] size = 16 $ss5print_9separator10terminatoryypd_S2StFTm
func[24949] size = 144 main
Total size: 160
```


## How it works

This tool analyzes functions in `.wasm` program and estimates the stack allocation size for each function by emulating stack pointer operations.

LLVM generates "shadow stack" to put local variables referenced through pointers for C-family languages. The stack pointer for the shadow stack `__stack_pointer` is stored in the `global` space of the `.wasm` program. Usually stored in the `globals[0]` since other language features rarely use `global` space.

This tool emulates the WebAssembly instructions between reading `__stack_pointer` and writing back to it.
