use crate::prelude::*;

use std::collections::hash_map::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;

use objectstore::{Identifier, OPath};

//PLANNED: may become a disk backed implementation since this can become big
#[derive(Debug)]
pub(crate) struct Entry {
    identifier: Identifier,
}

impl Entry {
    pub(crate) fn as_identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub(crate) fn to_opath(&self) -> OPath {
        self.identifier.to_opath()
    }
}

pub(crate) struct InodeDb {
    //PLANNED: caches
    inode_to_identifier: Mutex<HashMap<u64, Arc<Entry>>>,
}

impl InodeDb {
    pub fn new() -> Result<InodeDb> {
        Ok(InodeDb {
            inode_to_identifier: Mutex::new(HashMap::new()),
        })
    }

    pub fn store(&mut self, inode: u64, identifier: Identifier) -> Arc<Entry> {
        let mut inode_to_identifier = self.inode_to_identifier.lock();

        let entry = Arc::new(Entry {
            identifier: identifier,
        });

        inode_to_identifier.insert(inode, Arc::clone(&entry));
        entry
    }

    pub fn get(&mut self, inode: u64) -> Option<Arc<Entry>> {
        let mut inode_to_identifier = self.inode_to_identifier.lock();
        //PLANNED: touch/refresh self.inodedb caches
        if let Some(entry) = inode_to_identifier.get(&inode) {
            Some(Arc::clone(entry))
        } else {
            None
        }
    }
}
