// A simple ckb-vm profiler with debugger syscall implemented.

use ckb_vm::{
    machine::{
        asm::{AsmCoreMachine, AsmMachine},
        DefaultMachineBuilder, VERSION1,
    },
    registers::{A0, A7},
    Bytes, Error as VMError, Memory, Register, SupportMachine, Syscalls,
};
use clap::{crate_version, App, Arg};
use std::env;
use std::process::exit;

pub struct Debugger {}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {}
    }
}

impl<Mac: SupportMachine> Syscalls<Mac> for Debugger {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), VMError> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, VMError> {
        let number = machine.registers()[A7].to_u64();
        if number != 2177 {
            return Ok(false);
        }

        let mut addr = machine.registers()[A0].to_u64();
        let mut buffer = Vec::new();

        loop {
            let byte = machine
                .memory_mut()
                .load8(&Mac::REG::from_u64(addr))?
                .to_u8();
            if byte == 0 {
                break;
            }
            buffer.push(byte);
            addr += 1;
        }

        let s = String::from_utf8(buffer).expect("utf8 error");
        println!("{}", s);

        Ok(true)
    }
}

fn main() {
    drop(env_logger::init());
    let matches = App::new("ckb-signal-profiler")
        .version(crate_version!())
        .arg(
            Arg::with_name("bin")
                .long("bin")
                .help("Executable file in RISC-V. Must contain debug information.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .help("Output profiling file. Will be used by `pprof` command line.")
                .default_value("simple.profile")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("frequency")
                .long("fre")
                .short("f")
                .help("Sampling frequency, per second. Suggested value between 99~199")
                .default_value("199")
                .takes_value(true),
        )
        .arg(Arg::with_name("args").multiple(true))
        .get_matches();

    let matches_bin = matches.value_of("bin").unwrap();
    let matches_frequency = matches.value_of("frequency").unwrap();
    let matches_output = matches.value_of("output").unwrap();

    let matches_args = matches.values_of("args").unwrap_or_default();
    let riscv_args0: Vec<String> = matches_args.into_iter().map(|s| s.clone().into()).collect();
    let riscv_args: Vec<Bytes> = riscv_args0.into_iter().map(|s| s.into()).collect();
    let code = std::fs::read(matches_bin).unwrap().into();
    let frequency = matches_frequency.parse::<i32>().unwrap();

    let asm_core = AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
        VERSION1,
        u64::MAX,
    );
    let core = DefaultMachineBuilder::new(asm_core)
        .syscall(Box::new(Debugger::new()))
        .build();
    let mut machine = Box::pin(AsmMachine::new(core, None));

    ckb_vm_signal_profiler::start_profiler(matches_output, &machine, &code, frequency)
        .expect("profiler start failure");

    machine.load_program(&code, &riscv_args).unwrap();
    let result = machine.run();
    if result != Ok(0) {
        println!("Error result: {:?}", result);
        exit(i32::from(result.unwrap_or(-1)));
    }
    ckb_vm_signal_profiler::stop_profiler().expect("profiler start failure");
}
