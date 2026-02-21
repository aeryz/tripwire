use crate::runner::{WasmRunner, WasmVM};

pub mod runner;

struct SimpleVM;

impl WasmVM for SimpleVM {
    const ALLOC_FN_NAME: &str = "alloc";

    const MEMORY_NAME: &str = "memory";

    type Data = ();

    fn define_imports(linker: &mut wasmtime::Linker<Self::Data>) -> anyhow::Result<()> {
        linker.func_wrap("host", "debug", |param: i32| {
            println!("[WASM] {param}");
        })?;

        Ok(())
    }
}

fn main() {
    load_and_run_wasm().unwrap();
}

fn load_and_run_wasm() -> anyhow::Result<()> {
    let mut wasm_runner = WasmRunner::<SimpleVM>::load(env!("WASM_BINARY_PATH"), ()).unwrap();

    let x1 = wasm_runner.write_bytes(b"Hello, ")?;
    let y1 = wasm_runner.write_bytes(b"wasm!")?;

    let _ = wasm_runner
        .instance
        .get_typed_func::<(u32, u32, u32, u32, u32, u32), u32>(
            &mut wasm_runner.store,
            "entrypoint",
        )?
        .call(
            &mut wasm_runner.store,
            (x1.ptr, x1.len, y1.ptr, y1.len, 40, 2),
        )?;

    Ok(())
}
