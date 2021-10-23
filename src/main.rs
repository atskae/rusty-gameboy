mod cpu_core;

use crate::cpu_core::cpu::Cpu;
use log::{debug, info};

fn main() {
    env_logger::init();
    info!("Starting rusty-gameboy 🦀🎮");
    let cpu = Cpu::new();
    debug!("Created a CPU object {}", cpu);
}
