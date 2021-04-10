#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
mod prelude;
use crate::prelude::*;

use clap::ArgMatches;

mod optargs;
pub use self::optargs::optargs;
extern crate lazy_static;

mod errors;
pub mod identifier;
mod identifier_kind;
mod object;
pub mod objectstore;
mod opath;
mod rev_cursor;

mod init;
mod mkdir;
mod show;

/// Objectstore version
pub const VERSION: u32 = 0;

/// Prefix used for symlinks to uberallfs objects
pub const VERSION_PREFIX: [u8; 13] = *b"//uberallfs//";

pub fn cmd(matches: &ArgMatches) -> Result<()> {
    let dir = matches.value_of_os("DIRECTORY").expect("infallible");

    trace!("dir: {:?}", dir);

    match matches.subcommand() {
        ("init", Some(sub_m)) => init::opt_init(dir, sub_m),
        ("mkdir", Some(sub_m)) => mkdir::opt_mkdir(dir, sub_m),
        ("show", Some(sub_m)) => show::opt_show(dir, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}
