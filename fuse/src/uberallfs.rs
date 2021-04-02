use crate::prelude::*;

use std::path::Path;

use std::ffi::OsStr;

use objectstore::objectstore::ObjectStore;

use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};

pub struct UberallFS {
    objectstore: ObjectStore,
}

impl UberallFS {
    pub fn new(objectstore_dir: &Path) -> Result<UberallFS> {
        Ok(UberallFS {
            objectstore: ObjectStore::open(objectstore_dir)?,
        })
    }

    pub fn mount(
        self,
        mountpoint: &Path,
        offline: bool,
        root: Option<&OsStr>,
        _options_planned: Option<Vec<String>>,
    ) -> Result<()> {
        let mut options = vec![
            MountOption::RO,
            MountOption::FSName("uberallfs".to_string()),
        ];
        options.push(MountOption::AutoUnmount); //TODO: optarg?
        fuser::mount2(self, mountpoint, &options)?;
        Ok(())
    }
}

impl Filesystem for UberallFS {



}
