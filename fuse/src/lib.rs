mod prelude;
use uberall::clap::{self, ArgMatches};

use crate::prelude::*;

mod optargs;
pub use self::optargs::optargs;

mod handledb;
mod inodedb;
mod mount;
mod uberallfs;

use handledb::HandleDb;
use inodedb::InodeDb;

pub const VERSION: u32 = 0;

pub fn cmd(matches: &ArgMatches) -> Result<()> {
    let mountpoint = matches.value_of_os("MOUNTPOINT");

    let mountpoint = mountpoint.unwrap();

    match matches.subcommand() {
        ("mount", Some(sub_m)) => mount::opt_mount(mountpoint, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}
