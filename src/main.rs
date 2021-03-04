#[macro_use]
extern crate clap;
use clap::{AppSettings, ArgMatches};

use std::io;

mod optargs;
pub use self::optargs::uberallfs_optargs;

use objectstore;

extern crate log;
use log::LevelFilter;

use simple_logger::SimpleLogger;

fn main() -> io::Result<()> {
    let matches = uberallfs_optargs()
        .setting(AppSettings::SubcommandRequired)
        .subcommand(objectstore::optargs())
        .get_matches();

    init_logging(&matches);

    match matches.subcommand() {
        ("objectstore", Some(sub_m)) => objectstore::cmd(sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
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
