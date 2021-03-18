mod prelude;
use crate::prelude::*;

use clap::ArgMatches;

mod optargs;
pub use self::optargs::optargs;

pub const VERSION: u16 = 0;

pub fn cmd(matches: &ArgMatches) -> Result<()> {
    let mountpoint = matches.value_of_os("MOUNTPOINT");

    let objectstore = matches.value_of_os("OBJECTSTORE").or(mountpoint).unwrap();
    let mountpoint = mountpoint.unwrap();

    trace!("mountpoint: {:?}", mountpoint);
    trace!("objectstore: {:?}", objectstore);

    match matches.subcommand() {
        //("init", Some(sub_m)) => init::opt_init(dir, sub_m),
        (name, _) => {
            unimplemented!("subcommand '{}'", name)
        }
    }
}
