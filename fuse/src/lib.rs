mod prelude;
use crate::prelude::*;

use clap::ArgMatches;

mod optargs;
pub use self::optargs::optargs;

mod mount;
mod uberallfs;
mod inodedb;

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
