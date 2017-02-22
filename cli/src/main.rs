extern crate clap;
extern crate lazybox;

mod runner;

use std::path::Path;
use clap::{Arg, App, SubCommand};
use runner::Runner;

fn main() {
    let matches = App::new("The lazybox cli tool")
        .version("0.1")
        .about("Utility tool for game management")
        .subcommand(SubCommand::with_name("run")
            .about("run the game")
            .arg(Arg::with_name("game")
                .short("g")
                .help("path to the game library")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("config")
                .short("c")
                .help("path to the game config")
                .required(true)
                .takes_value(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let game_path = matches.value_of("game").unwrap();
        let config_path = matches.value_of("config").unwrap();
        handle_run_command(Path::new(game_path), Path::new(config_path))
    }
}

fn handle_run_command(_game_path: &Path, config_path: &Path) {
    let runner = Runner::new(config_path);
    runner.run();
}
