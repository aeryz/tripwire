use crate::wasm_runner::{WasmRunner, WasmVM};

pub mod wasm_runner;

struct SimpleWasmVM;

impl WasmVM for SimpleWasmVM {
    const ALLOC_FN_NAME: &str = "alloc";

    const MEMORY_NAME: &str = "memory";

    type Data = ();
}

fn main() {
    let wasm_runner = WasmRunner::<SimpleWasmVM>::load(env!("WASM_BINARY_PATH"), ()).unwrap();
}
