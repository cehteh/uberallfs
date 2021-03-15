#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
use clap::ArgMatches;
use std::io;

mod optargs;
pub use self::optargs::optargs;

mod identifier;
mod identifier_kind;
mod init;
mod object;
mod objectstore;
mod rev_cursor;

extern crate log;

#[allow(unused_imports)]
use log::{debug, error, info, trace};

pub const VERSION: u16 = 0;

pub fn cmd(matches: &ArgMatches) -> io::Result<()> {
    let dir = matches.value_of_os("DIRECTORY").unwrap();

    trace!("dir: {:?}", dir);

    match matches.subcommand() {
        ("init", Some(sub_m)) => init::opt_init(dir, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}
