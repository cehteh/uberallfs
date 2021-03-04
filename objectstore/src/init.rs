use clap::ArgMatches;
use std::ffi::OsStr;
use std::fs::DirBuilder;
use std::io;

use log::{debug, error, info, trace};

pub(crate) fn init(dir: &OsStr, matches: &ArgMatches) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(dir)?;

    //    trace!("{:?}", ok);
    // if exists / force /mkdir
    //mkdir
    // if failed then if exists and force
    //   and it is a objectdir

    // initialize

    // unpack and verify import

    // or create a new root  --local or --public

    Ok(())
}
