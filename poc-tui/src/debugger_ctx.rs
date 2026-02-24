use capstone::prelude::*;
use color_eyre::eyre;
use nix::{
    sys::{
        ptrace,
        wait::{WaitStatus, waitpid},
    },
    unistd::Pid,
};
use std::{ffi::c_void, process::Command};

use crate::function_mapping::FunctionMapping;

pub const WASM_MEMORY_IMAGE_IDENT: &str = "wasm-memory-image";

#[derive(Debug)]
pub struct DebuggerCtx {
    pub pid: Pid,
    pub function_mapping: Option<FunctionMapping>,
}

impl DebuggerCtx {
    pub fn run_command(&mut self, command: &str) -> eyre::Result<()> {
        let child = Command::new(command).spawn()?;
        let ptrace_pid = Pid::from_raw(child.id() as i32);
        self.pid = ptrace_pid;
        ptrace::seize(ptrace_pid, ptrace::Options::PTRACE_O_TRACESYSGOOD)?;
        ptrace::interrupt(ptrace_pid)?;
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

        Ok(())
    }

    pub fn parse_perfmap(&mut self, bin_name: &str) -> eyre::Result<()> {
        self.function_mapping = Some(FunctionMapping::generate_from_perfmap_file_with_pid(
            bin_name,
            self.pid.as_raw() as u32,
        )?);

        Ok(())
    }

    pub fn disassemble(&self, index: usize) -> eyre::Result<String> {
        let Some(mapping) = &self.function_mapping else {
            return Ok("".into());
        };

        let (_, meta) = (&mapping).into_iter().skip(index).next().unwrap();

        let mut buf = Vec::new();
        let mut read_len = meta.size / size_of::<usize>() as u64 + 1;
        if meta.size % (size_of::<usize>() as u64) == 0 {
            read_len -= 1;
        }

        for i in 0..read_len {
            let read_data = ptrace::read(self.pid, (meta.addr + (8 * i)) as *mut c_void).unwrap();

            buf.extend_from_slice(&read_data.to_le_bytes());
        }

        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .expect("Failed to create Capstone object");

        let mut read_size = 0;
        let mut disas_str = String::new();

        let mut disas_iter = cs.disasm_iter(&buf, 0).unwrap();

        while let Some(instr) = disas_iter.next() {
            if read_size >= meta.size {
                break;
            }
            read_size += instr.len() as u64;
            disas_str += &format!("{instr}\n");
        }

        Ok(disas_str)
    }
}
