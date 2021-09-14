use crate::prelude::*;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use uberall::clap::ArgMatches;

use std::ffi::OsStr;

use crate::objectstore::ObjectStore;

pub(crate) fn opt_show(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let objectstore = ObjectStore::open(dir.as_ref())?;

    let path = matches
        .value_of_os("PATH")
        .or_else(|| Some(OsStr::from_bytes(b"/")));

    let (src, remaining) = objectstore.path_lookup(&path.map(PathBuf::from).unwrap(), None)?;

    if remaining.as_os_str().is_empty() {
        println!("{:?} -> {:?}", path.unwrap(), src);
    } else {
        println!("remaining {:?}", remaining);

        return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }
    Ok(())
}
