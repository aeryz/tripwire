// use wasmtime::Caller;

// use crate::wasm_runner::WasmVM;

// struct CosmwasmVM;

// impl WasmVM for CosmwasmVM {
//     const ALLOC_FN_NAME: &str = "alloc";

//     const MEMORY_NAME: &str = "memory";

//     type Data = ();

//     fn define_imports(linker: &mut wasmtime::Linker<Self::Data>) -> anyhow::Result<()> {
//         linker.func_wrap("env", "abort", |_: Caller<'_, ()>, _: i32| {})?;
//         linker.func_wrap("env", "db_next_key", |_: Caller<'_, ()>, _: i32| -> i32 {
//             0
//         })?;
//         linker.func_wrap("env", "db_next_value", |_: Caller<'_, ()>, _: i32| -> i32 {
//             0
//         })?;
//         linker.func_wrap("env", "db_next", |_: Caller<'_, ()>, _: i32| -> i32 { 0 })?;
//         linker.func_wrap(
//             "env",
//             "addr_humanize",
//             |_: Caller<'_, ()>, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap("env", "addr_validate", |_: Caller<'_, ()>, _: i32| -> i32 {
//             0
//         })?;
//         linker.func_wrap(
//             "env",
//             "addr_canonicalize",
//             |_: Caller<'_, ()>, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap(
//             "env",
//             "bls12_381_hash_to_g1",
//             |_: Caller<'_, ()>, _: i32, _: i32, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap(
//             "env",
//             "bls12_381_hash_to_g2",
//             |_: Caller<'_, ()>, _: i32, _: i32, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap(
//             "env",
//             "bls12_381_aggregate_g1",
//             |_: Caller<'_, ()>, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap(
//             "env",
//             "bls12_381_aggregate_g2",
//             |_: Caller<'_, ()>, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap(
//             "env",
//             "bls12_381_pairing_equality",
//             |_: Caller<'_, ()>, _: i32, _: i32, _: i32, _: i32| -> i32 { 0 },
//         )?;
//         linker.func_wrap("env", "debug", |_: Caller<'_, ()>, _: i32| {})?;
//         linker.func_wrap("env", "query_chain", |_: Caller<'_, ()>, _: i32| -> i32 {
//             0
//         })?;
//         linker.func_wrap("env", "db_read", |_: Caller<'_, ()>, _: i32| -> i32 { 0 })?;
//         linker.func_wrap("env", "db_write", |_: Caller<'_, ()>, _: i32, _: i32| {})?;
//         linker.func_wrap("env", "db_remove", |_: Caller<'_, ()>, _: i32| {})?;

//         Ok(())
//     }
// }
