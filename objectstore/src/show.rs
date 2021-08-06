use crate::prelude::*;

use clap::ArgMatches;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use std::ffi::OsStr;

use crate::objectstore::ObjectStore;

pub(crate) fn opt_show(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let mut objectstore = ObjectStore::open(Path::new(dir))?;

    let path = matches
        .value_of_os("PATH")
        .or_else(|| Some(OsStr::from_bytes(b"/")));

    let (src, remaining) = objectstore.path_lookup(&path.map(PathBuf::from).unwrap(), None)?;

    if remaining.as_os_str().is_empty() {
        println!("{:?} -> {:?}", path.unwrap(), src);
    } else {
        println!("could not resolve: {:?} ", remaining.unwrap());
    }
    Ok(())
}
