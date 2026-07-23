pub mod bit;
mod cpu;

use crate::cpu::CPU;

fn main() {
    println!("Hello, world!");

    let mut cpu = CPU::new();
    cpu.interpret(vec![]);
}
