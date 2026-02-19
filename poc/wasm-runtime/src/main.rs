use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};

use interprocess::local_socket::{GenericFilePath, prelude::*};

use crate::runner::{WasmRunner, WasmVM};

pub mod runner;

struct SimpleVM;

impl WasmVM for SimpleVM {
    const ALLOC_FN_NAME: &str = "alloc";

    const MEMORY_NAME: &str = "memory";

    type Data = ();

    fn define_imports(linker: &mut wasmtime::Linker<Self::Data>) -> anyhow::Result<()> {
        linker.func_wrap("host", "debug", |param: i32| {
            println!("param {param}");
        })?;

        Ok(())
    }
}

fn main() {
    load_and_run_wasm().unwrap();
}

fn load_and_run_wasm() -> anyhow::Result<()> {
    println!("here1");
    let mut wasm_runner = WasmRunner::<SimpleVM>::load(env!("WASM_BINARY_PATH"), ()).unwrap();

    let x1 = wasm_runner.write_bytes(b"Hello, ")?;
    let y1 = wasm_runner.write_bytes(b"wasm!")?;
    println!("here2");

    let socket = format!("/tmp/{}.sock", std::process::id())
        .to_fs_name::<GenericFilePath>()
        .unwrap();
    println!("here3");

    let mut retries = 10;
    while retries > 0 {
        std::thread::sleep(Duration::from_secs(3));
        println!("trying to connect..");
        if let Ok(conn) = LocalSocketStream::connect(socket.clone()) {
            println!("connected");
            let mut conn = BufReader::new(conn);
            conn.get_mut().write_all(b"Hello from client!\n").unwrap();
            let mut buf = String::new();
            conn.read_line(&mut buf).unwrap();

            println!("got the go signal: {buf}");

            break;
        }

        std::thread::sleep(Duration::from_secs(1));
        retries -= 1;
    }

    println!("somehow weirdly here????");

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
