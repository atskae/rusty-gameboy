mod cli;
mod cpu_core;

use crate::cpu_core::cpu::Cpu;
use cli::CommandLineArgs;
use log::{debug, info};

fn main() {
    env_logger::init();
    info!("Starting rusty-gameboy 🦀🎮");
    let args = CommandLineArgs::new();
    debug!("Command line args: {:?}", args);

    let mut cpu = Cpu::new(args.rom_path);
    debug!("Created a CPU object {}", cpu);
    cpu.start(args.subcommand);
}
