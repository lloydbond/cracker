extern crate getopts;
use getopts::Options;

use crate::utils;

const PROGRAM_DESC: &str = env!("CARGO_PKG_DESCRIPTION");
const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!(
        "Welcome to {}\n{}\n{}",
        PROGRAM_NAME,
        PROGRAM_DESC,
        opts.usage(&brief)
    );
}

pub fn parse_args(args: &[String]) -> Result<String, utils::Error> {
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(String::new());
    }
    let filename = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        "Makefile".to_string()
    };
    Ok(filename)
}
