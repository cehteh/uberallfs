use crate::prelude::*;

use clap::ArgMatches;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use std::ffi::OsStr;

use crate::objectstore::ObjectStore;
use crate::opath::OPath;

pub(crate) fn opt_show(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let mut objectstore = ObjectStore::open(Path::new(dir))?;

    let path = matches
        .value_of_os("PATH")
        .or_else(|| Some(OsStr::from_bytes(b"/")));

    let (src, remaining) = objectstore.path_lookup(path.map(|f| OPath::prefix(f)), None)?;

    if remaining == Some(OPath::new()) {
        println!("{:?} -> {:?}", path.unwrap(), src);
    } else {
        println!("could not resolve: {:?} ", remaining.unwrap());
    }
    Ok(())
}
