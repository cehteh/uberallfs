#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
mod prelude;
use crate::prelude::*;

use clap::ArgMatches;

mod optargs;
pub use self::optargs::optargs;
extern crate lazy_static;

mod errors;
mod handle;
mod identifier;
mod identifier_kind;
mod object;
mod objectpath;
mod objectstore;
mod permissions;
mod rev_cursor;
mod vfs;

mod init;
mod mkdir;
mod show;

pub use crate::objectpath::ObjectPath;
pub use crate::objectstore::{ObjectStore, SubObject};
pub use handle::Handle;
pub use identifier::{Flipbase64, Identifier, IdentifierBin};
pub use identifier_kind::{Mutability, ObjectType, SharingPolicy};
pub use object::Object;
pub use permissions::{PermissionCheck, PermissionController};

pub use vfs::VirtualFileSystem;

//PLANNED: mockup types defined and exported that dont have a implementation yet
pub type UserId = u32;

/// Objectstore version
pub const VERSION: u32 = 0;

/// Prefix used for symlinks to uberallfs objects
pub const RESERVED_PREFIX: [u8; 11] = *b".uberallfs.";

pub fn cmd(matches: &ArgMatches) -> Result<()> {
    let dir = matches.value_of_os("DIRECTORY").expect("infallible");

    trace!("objectstore directory: {:?}", dir);

    match matches.subcommand() {
        ("init", Some(sub_m)) => init::opt_init(dir, sub_m),
        ("mkdir", Some(sub_m)) => mkdir::opt_mkdir(dir, sub_m),
        ("show", Some(sub_m)) => show::opt_show(dir, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}
