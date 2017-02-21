extern crate clap;
extern crate lazybox;

mod project;

use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("The lazybox cli tool")
                        .version("0.1")
                        .about("Utility tool for game management")
                        .subcommand(SubCommand::with_name("run")
                                    .about("run the game"))
                        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        project::Runner::new().run()
    }
}