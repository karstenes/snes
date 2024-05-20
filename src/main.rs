mod cpu;

use cpu::CPU;
use std::process::ExitCode;

#[allow(non_snake_case)]

pub struct Console{
    cpu: CPU
}

fn main() -> ExitCode {
    let _test = CPU::new();
    let _test2 = _test.A;

    ExitCode::SUCCESS
}
