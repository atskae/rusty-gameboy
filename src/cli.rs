use clap::{load_yaml, App};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Subcommand {
    Run,
    Disassemble,
}

#[derive(Debug)]
pub struct CommandLineArgs {
    pub subcommand: Subcommand,
    pub rom_path: PathBuf,
}

impl CommandLineArgs {
    pub fn new() -> CommandLineArgs {
        let cli_yml_file = load_yaml!("cli.yml");
        let matches = App::from_yaml(cli_yml_file).get_matches();

        let subcommand = match matches.subcommand_name() {
            Some("run") => Subcommand::Run,
            Some("disassemble") => Subcommand::Disassemble,
            _ => unreachable!(),
        };

        let rom_path = PathBuf::from(matches.value_of("rom").unwrap());

        CommandLineArgs {
            subcommand,
            rom_path,
        }
    }
}
