# Rust <-> assemblyscript example

This repository showcases the following:

1. How to ompile example assemblyscript to wasm 
    - it has a function `transform` that takes string and returns didderent string 
    - it has a host defined function called `log` that sends a string to rust side for logging
2. How to use WebAssembly runtime (wasmer) to setup and call this module
    - call wasm module change with http response from some json web api
    - print the output in rust side

## Running instruction

Prerequisities:

- nodejs
- nix
- cargo

And then run:

```bash
cd transformation
npm install
npm run asbuild # this will create required wasm binaries

cd ..

# run the host side
cargo run
   Compiling wasm v0.1.0 (/home/flakm/programming/modivo/wasm)
    Finished dev [unoptimized + debuginfo] target(s) in 2.54s
     Running `target/debug/wasm`
Compiling module...
Creating the imported function...
[0] my_response: status: 200, body: {"name":"John", "age":30}
[1] my_response: status: 200, body: {"tag":"@John"}
```


## TODOS

- Only basic testing support there is other project `as-pect` that might be [awesome](https://as-pect.gitbook.io/as-pect/)
- assemblyscript has to be manually installed and run because it's not packaged in nix
- cargo crane project is super basic example
- better error handling: https://docs.wasmer.io/integrations/examples/errors

## Pain points

- Error handling is painful since we loose a lot of information when code throws error during serialization [issue](https://stackoverflow.com/questions/75359487/handling-exception-for-invalid-json-format-for-assemblyscript-json-in-json-pa)
- Sharing memory is [complicated](https://docs.wasmer.io/integrations/examples/memory) for example if we want to provide host functions to wasm module
  that will reach back to some memory in host from the guest it will require dereferencing a raw pointer that is leaked (kind of like with FFI) or some kind of hash map backed storage.



## Interesting material


1. Wasmer rust - rust example: https://github.com/wasmerio/wasmer-rust-example/blob/master/src/main.rs
    
   wasmer is an open-source runtime for executing WebAssembly on the Server. 

2. AssemblyScript A TypeScript-like language for WebAssembly  https://www.assemblyscript.org/

- [Example of importing functions into wasm](https://wasmbyexample.dev/examples/importing-javascript-functions-into-webassembly/importing-javascript-functions-into-webassembly.assemblyscript.en-us.html)

3. `wasm-bindgen` rust crate enabling high level interactions between wasm modules
   allows for similar code:

```rust
use wasm_bindgen::prelude::*;

// Import the `window.alert` function from the Web.
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Export a `greet` function from Rust to JavaScript, that alerts a
// hello message.
#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
```


## Set up transformation

https://www.assemblyscript.org/getting-started.html#setting-up-a-new-project

