mod compile;

pub use compile::*;

use clap::{builder::ValueParser, Arg, Command};

pub fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(false)
        .arg(
            Arg::new("input")
                .value_parser(ValueParser::path_buf())
                .required(false),
        )
        .arg(
            Arg::new("output")
                .value_parser(ValueParser::path_buf())
                .required(false),
        )
}
