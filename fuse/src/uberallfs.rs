use crate::prelude::*;

use std::collections::hash_map::HashMap;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use objectstore::identifier::Identifier;
use objectstore::objectstore::{OPath, ObjectStore};

use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, Request,
};

//PLANNED: may become a disk backed implementation since this can become big
#[derive(Debug)]
struct InodeDBEntry {
    identifier: Identifier,
}

struct InodeDB {
    inode_to_identifier: HashMap<u64, InodeDBEntry>,
}

impl InodeDB {
    pub fn new() -> Result<InodeDB> {
        Ok(InodeDB {
            inode_to_identifier: HashMap::new(),
        })
    }

    pub fn store(&mut self, inode: u64, identifier: Identifier) {
        self.inode_to_identifier
            .insert(inode, InodeDBEntry { identifier });
    }

    pub fn find(&mut self, inode: u64) -> Option<&InodeDBEntry> {
        self.inode_to_identifier.get(&inode)
    }
}

pub struct UberallFS {
    objectstore: ObjectStore,
    inodedb: InodeDB,
}

impl UberallFS {
    pub fn new(objectstore_dir: &Path) -> Result<UberallFS> {
        Ok(UberallFS {
            objectstore: ObjectStore::open(objectstore_dir)?,
            inodedb: InodeDB::new()?,
        })
    }

    pub fn mount(
        mut self,
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

        let (identifier, none) = self.objectstore.path_lookup(root.map(From::from), None)?;
        assert_eq!(none, None);

        self.inodedb.store(1, identifier);
        fuser::mount2(self, mountpoint, &options)?;
        Ok(())
    }
}

impl Filesystem for UberallFS {



}
