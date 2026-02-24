use std::{os::raw::c_void, process::Command};

use anyhow::anyhow;
use nix::{
    sys::{
        ptrace,
        wait::{WaitStatus, waitpid},
    },
    unistd::Pid,
};

use crate::perf_util::FunctionMapping;

pub mod cosmwasm;
pub mod perf_util;

pub const WASM_MEMORY_IMAGE_IDENT: &str = "wasm-memory-image";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let child = Command::new(env!("WASM_RUNTIME_PATH")).spawn().unwrap();

    let child_pid = child.id();
    let ptrace_pid = Pid::from_raw(child_pid as i32);
    ptrace::seize(ptrace_pid, ptrace::Options::PTRACE_O_TRACESYSGOOD).unwrap();
    println!("child pid: {child_pid}");

    ptrace::interrupt(ptrace_pid).unwrap();

    match waitpid(ptrace_pid, None)? {
        WaitStatus::Stopped(_, _) | WaitStatus::PtraceEvent(..) => {}
        other => eprintln!("wait: {other:?}"),
    }

    loop {
        ptrace::syscall(ptrace_pid, None)?;

        match waitpid(ptrace_pid, None)? {
            WaitStatus::PtraceSyscall(ptrace_pid) => {
                let syscall = ptrace::syscall_info(ptrace_pid)?;
                if syscall.op == libc::PTRACE_SYSCALL_INFO_ENTRY {
                    let syscall = unsafe { syscall.u.entry };

                    // wasmtime uses `memfd_create` to create an anonymous in-memory file. This happens
                    // after the `perf` is written under `/tmp/perf-PID.map` and before executing the
                    // WASM binary. This means we can inject our traps right at this moment.
                    //
                    // https://github.com/bytecodealliance/wasmtime/blob/ee7e125309f5bc784b8feb8969261ae41fb4703b/crates/wasmtime/src/runtime/vm/sys/unix/vm.rs#L120
                    if syscall.nr == libc::SYS_memfd_create as u64 {
                        let mut memory_name: Vec<u8> = Vec::new();
                        let mut len = (WASM_MEMORY_IMAGE_IDENT.len() / size_of::<usize>()) + 1;
                        if WASM_MEMORY_IMAGE_IDENT.len() % size_of::<usize>() == 0 {
                            len -= 1;
                        }
                        for i in 0..len {
                            let data = ptrace::read(
                                ptrace_pid,
                                ((syscall.args[0] as usize) + (i * size_of::<usize>()))
                                    as *mut c_void,
                            )?;
                            memory_name
                                .extend(data.to_le_bytes().iter().take_while(|i| *i != &0u8));
                        }

                        if &memory_name == WASM_MEMORY_IMAGE_IDENT.as_bytes() {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let function_mapping =
        FunctionMapping::generate_from_perfmap_file_with_pid("wasm_binary", child_pid as u32)?;

    let named_fn = function_mapping
        .get_function("trim_ascii_whitespace")
        .unwrap();

    println!("named_fn: {:x}", named_fn.addr);

    println!("inserting the trap");

    // Insert the trap to `trim_ascii_whitespace`
    let original_word = ptrace::read(ptrace_pid, named_fn.addr as *mut c_void)? as u64;
    // Note that ptrace writes one word, *NOT* one byte. Hence, we are reading
    // the whole word first. Then changing the one byte we wanna modify.
    let trapped_word = ((original_word & !0xFF) | (0xCC as u64)) as i64;
    ptrace::write(ptrace_pid, named_fn.addr as *mut c_void, trapped_word)?;

    println!("trap inserted to `trim_ascii_whitespace`");

    ptrace::cont(ptrace_pid, None)?;

    match waitpid(ptrace_pid, None)? {
        WaitStatus::Stopped(_, _) | WaitStatus::PtraceEvent(..) => {
            println!("!! hit the trap !!");
        }
        other => return Err(anyhow!("unexpected: {other:?}")),
    }

    let mut regs = ptrace::getregs(ptrace_pid)?;
    regs.rip -= 1;

    let rip = regs.rip;
    ptrace::write(ptrace_pid, regs.rip as *mut c_void, original_word as i64)?;

    ptrace::setregs(ptrace_pid, regs)?;

    // We step one instruction so that we can write back the trap
    ptrace::step(ptrace_pid, None)?;

    match waitpid(ptrace_pid, None)? {
        WaitStatus::Stopped(_, _) | WaitStatus::PtraceEvent(..) => {}
        other => return Err(anyhow!("unexpected: {other:?}")),
    }

    ptrace::write(ptrace_pid, rip as *mut c_void, trapped_word)?;

    ptrace::cont(ptrace_pid, None)?;

    match waitpid(ptrace_pid, None)? {
        WaitStatus::Stopped(_, _) | WaitStatus::PtraceEvent(..) => {
            println!("!! hit the second trap !!");
        }
        other => return Err(anyhow!("unexpected: {other:?}")),
    }

    let mut regs = ptrace::getregs(ptrace_pid)?;
    regs.rip -= 1;

    ptrace::write(ptrace_pid, regs.rip as *mut c_void, original_word as i64)?;

    ptrace::setregs(ptrace_pid, regs)?;

    ptrace::cont(ptrace_pid, None)?;

    Ok(())
}
