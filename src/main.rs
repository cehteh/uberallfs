#[macro_use]
extern crate clap;
use clap::{Arg, App};

mod args;
pub use self::args::uberallfs_args;


use objectstore;

#[macro_use]
extern crate log;
use log::{debug, error, info, log_enabled, Level, LevelFilter};

use simple_logger::SimpleLogger;

fn main() {
    let matches = uberallfs_args()
        .subcommand(objectstore::args())
        .get_matches();

    init_logging(&matches);
}

fn init_logging(matches: &ArgMatches) {
    let mut verbosity_level = 1;

    if matches.is_present("quiet") {
        verbosity_level = 0
    }
    if matches.is_present("debug") {
        verbosity_level = 4
    }

    verbosity_level += matches.occurrences_of("verbose");

    let verbosity_level = match verbosity_level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 | _ => LevelFilter::Trace,
    };

    SimpleLogger::new()
        .with_level(verbosity_level)
        .init()
        .unwrap();
}
