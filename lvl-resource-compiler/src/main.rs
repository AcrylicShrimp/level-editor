mod cli;
mod processors;

use cli::{cli, compile};
use log::{error, LevelFilter};
use std::path::PathBuf;

fn main() {
    env_logger::Builder::from_env("LOG")
        .filter_level(LevelFilter::Info)
        .format_module_path(false)
        .format_target(false)
        .init();

    let matches = cli().get_matches();

    match matches.subcommand() {
        None => {
            let input = matches.get_one::<PathBuf>("input");
            let output = matches.get_one::<PathBuf>("output");

            if let Err(err) = compile(input, output) {
                let mut errors = Vec::new();

                for cause in err.chain() {
                    errors.push(format!("- {}", cause.to_string()));
                }

                error!("failed to compile resources. error:\n{}", errors.join("\n"));
            }
        }
        _ => unreachable!(),
    }
}
