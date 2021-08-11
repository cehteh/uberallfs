#[macro_use]
extern crate clap;
use clap::{AppSettings, ArgMatches};
use std::error::Error;
use std::io;

use libc;

mod optargs;
pub use self::optargs::uberallfs_optargs;

extern crate log;
use log::LevelFilter;

use simple_logger::SimpleLogger;

#[cfg(unix)]
fn platform_init() {
    unsafe {
        // no 'other' access
        libc::umask(libc::S_IRWXO);
    }
}

fn error_to_exitcode(error: Box<dyn Error>) -> i32 {
    error
        .downcast::<io::Error>()
        .map_or(libc::EXIT_FAILURE, |e| {
            e.raw_os_error().unwrap_or(match e.kind() {
                io::ErrorKind::AlreadyExists => libc::EEXIST,
                _ => todo!("implement for kind {:?}", e.kind()),
            })
        })
}

fn main() {
    platform_init();
    let matches = uberallfs_optargs()
        .setting(AppSettings::SubcommandRequired)
        .subcommand(objectstore::optargs())
        .subcommand(fuse::optargs())
        .get_matches();

    init_logging(&matches);

    if let Err(err) = match matches.subcommand() {
        ("objectstore", Some(sub_m)) => objectstore::cmd(sub_m),
        ("fuse", Some(sub_m)) => fuse::cmd(sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    } {
        log::error!("Error: {}", &err);
        std::process::exit(error_to_exitcode(err));
    } else {
        log::info!("OK");
    }
}

fn init_logging(matches: &ArgMatches) {
    let mut verbosity_level = 1;

    if matches.is_present("quiet") {
        verbosity_level = 0
    }
    if matches.is_present("debug") {
        verbosity_level = 4;
    }

    verbosity_level += matches.occurrences_of("verbose");

    use log::LevelFilter::*;
    let verbosity_level = match verbosity_level {
        0 => Off,
        1 => Error,
        2 => Warn,
        3 => Info,
        4 => Debug,
        _ => Trace,
    };

    SimpleLogger::new()
        .with_level(verbosity_level)
        .init()
        .expect("Failed to initialize the logging System");
}
