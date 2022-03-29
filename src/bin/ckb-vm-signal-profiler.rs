// A simple ckb-vm profiler with debugger syscall implemented.

use ckb_vm::{
    machine::{
        asm::{AsmCoreMachine, AsmMachine},
        DefaultMachineBuilder, VERSION1,
    },
    registers::{A0, A7},
    Bytes, Error as VMError, Memory, Register, SupportMachine, Syscalls,
};
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
    env_logger::init();

    let args: Vec<String> = env::args().skip(1).collect();
    let code = std::fs::read(args[0].clone()).unwrap().into();
    let riscv_args: Vec<Bytes> = if args.len() > 2 {
        (&args[2..]).into_iter().map(|s| s.clone().into()).collect()
    } else {
        Vec::new()
    };

    let asm_core = AsmCoreMachine::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
        VERSION1,
        u64::MAX,
    );
    let core = DefaultMachineBuilder::new(asm_core)
        .syscall(Box::new(Debugger::new()))
        .build();
    let mut machine = Box::pin(AsmMachine::new(core, None));

    ckb_vm_signal_profiler::start_profiler("simple.profile", &machine, &code, 1000)
        .expect("profiler start failure");

    machine.load_program(&code, &riscv_args).unwrap();
    let result = machine.run();
    if result != Ok(0) {
        println!("Error result: {:?}", result);
        exit(i32::from(result.unwrap_or(-1)));
    }
    ckb_vm_signal_profiler::stop_profiler().expect("profiler start failure");
}
