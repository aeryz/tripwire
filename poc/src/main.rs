use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    os::raw::c_void,
    process::Command,
};

use interprocess::local_socket::{GenericFilePath, ListenerOptions, prelude::*};
use libc::{_SC_PAGESIZE, PROT_EXEC, PROT_READ, PROT_WRITE, SIGCONT, mprotect};
use nix::{
    errno::{self, Errno},
    sys::{
        ptrace::{self, AddressType, Request, RequestType},
        signal::Signal,
        wait::{WaitPidFlag, WaitStatus, waitpid},
    },
    unistd::Pid,
};
use tokio::signal;

use crate::perf_util::FunctionMapping;

pub mod cosmwasm;
pub mod perf_util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let child = Command::new("/home/aeryz/dev/tripwire/tripwire/target/debug/wasm-runtime")
        .spawn()
        .unwrap();

    let child_pid = child.id();
    let ptrace_pid = Pid::from_raw(child_pid as i32);
    ptrace::seize(ptrace_pid, ptrace::Options::empty()).unwrap();
    // ptrace::attach(ptrace_pid).unwrap();
    println!("child pid: {child_pid}");

    // let res = unsafe {
    //     libc::ptrace(
    //         Request::PTRACE_LISTEN as RequestType,
    //         libc::pid_t::from(ptrace_pid),
    //         std::ptr::null_mut() as AddressType,
    //         std::ptr::null_mut() as *mut c_void,
    //     )
    // };

    // if res != 0 {
    //     panic!("listen errno: {}", Errno::result(res).unwrap())
    // }

    let opts = ListenerOptions::new().name(
        format!("/tmp/{child_pid}.sock")
            .to_fs_name::<GenericFilePath>()
            .unwrap(),
    );

    let listener = opts.create_sync().unwrap();

    println!("waiting for the listener");
    std::thread::sleep(std::time::Duration::from_secs(10));
    for conn in listener.incoming() {
        let conn = conn.unwrap();

        let function_mapping =
            FunctionMapping::generate_from_perfmap_file_with_pid("wasm_binary", child_pid as u32)?;

        let named_fn = function_mapping
            .get_function("trim_ascii_whitespace")
            .unwrap();

        // for (k, v) in &function_mapping {
        //     println!("k: {}, v: {:?}", k, v);
        // }

        let page_size = unsafe { libc::sysconf(_SC_PAGESIZE) } as usize;

        let page_start = named_fn.addr as usize & !(page_size - 1);

        println!("named_fn: {:x} {:x}", named_fn.addr, page_start);

        println!("before interrupt");
        ptrace::interrupt(ptrace_pid).unwrap();

        match waitpid(ptrace_pid, None)? {
            WaitStatus::Stopped(_, _) | WaitStatus::PtraceEvent(..) => {}
            other => eprintln!("wait: {other:?}"),
        }
        println!("after interrupt");

        ptrace::write(
            Pid::from_raw(child_pid as i32),
            named_fn.addr as *mut c_void,
            0xCC,
        )
        .unwrap();
        println!("after write");

        std::thread::spawn(move || attach_ptrace(ptrace_pid).unwrap());

        ptrace::cont(ptrace_pid, Some(Signal::SIGCONT)).unwrap();

        let mut conn = BufReader::new(conn);

        std::thread::sleep(std::time::Duration::from_secs(2));

        let mut buffer = String::new();
        println!("tring to read");
        conn.read_line(&mut buffer).unwrap();
        println!("read: {buffer}");
        conn.get_mut().write_all(b"Hello from listener!\n").unwrap();
        println!("written hi");

        break;
    }

    println!("breakpoint placed");

    let ctrl_c = signal::ctrl_c();
    println!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    println!("Exiting...");

    Ok(())
}

fn attach_ptrace(pid: Pid) -> anyhow::Result<()> {
    match waitpid(pid, None) {
        Ok(WaitStatus::Stopped(_, Signal::SIGTRAP)) => {
            println!("got the sigtrap bro")
        }
        Ok(_) => {}
        Err(_) => panic!("oh boy"),
    }

    Ok(())
}

// extern "C" fn sigtrap_handler(_sig: i32, _info: *mut siginfo_t, uctx: *mut c_void) {
//     // WARNING: signal handlers must be async-signal-safe.
//     // Keep this minimal.

//     unsafe {
//         let uc = &mut *(uctx as *mut ucontext_t);

//         println!("im attached brotha");

//         #[cfg(target_arch = "x86_64")]
//         {
//             use libc::REG_RIP;

//             let rip = uc.uc_mcontext.gregs[REG_RIP as usize] as usize;
//             let bp_addr = rip - 1;

//             // push rbp
//             *(bp_addr as *mut u8) = 0x55;

//             uc.uc_mcontext.gregs[REG_RIP as usize] = bp_addr as i64;
//         }
//         // {
//         //     // RIP points *after* the INT3 byte
//         //     let rip = uc.uc_mcontext.gregs[REG_RIP as usize] as usize;
//         //     let bp_addr = rip - 1;

//         //     // Example: rewind RIP to re-execute original instruction later
//         //     uc.uc_mcontext.gregs[REG_RIP as usize] = bp_addr as i64;

//         //     // You would also restore original byte at bp_addr,
//         //     // set trap-flag for single-step, etc.
//         // }
//     }
// }

// pub fn install_sigtrap_handler() {
//     unsafe {
//         let mut sa: sigaction = mem::zeroed();
//         sa.sa_flags = SA_SIGINFO;
//         sa.sa_sigaction = sigtrap_handler as usize;
//         sigemptyset(&mut sa.sa_mask);

//         sigaction(SIGTRAP, &sa, std::ptr::null_mut());
//     }
// }
