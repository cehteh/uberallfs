use crate::prelude::*;

use std::collections::hash_map::HashMap;

use objectstore::identifier::Identifier;
use objectstore::opath::OPath;

//PLANNED: may become a disk backed implementation since this can become big
#[derive(Debug)]
pub(crate) struct InodeDBEntry {
    identifier: Identifier, //TODO: Arc<Identifier>
}

impl InodeDBEntry {
    pub(crate) fn as_identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub(crate) fn to_opath(&self) -> OPath {
        self.identifier.to_opath()
    }
}


pub(crate) struct InodeDB {
    //PLANNED: caches
    inode_to_identifier: HashMap<u64, InodeDBEntry>,
}

impl InodeDB {
    pub fn new() -> Result<InodeDB> {
        Ok(InodeDB {
            inode_to_identifier: HashMap::new(),
        })
    }

    pub fn store(&mut self, inode: u64, identifier: &Identifier) {
        self.inode_to_identifier.insert(
            inode,
            InodeDBEntry {
                identifier: identifier.clone(),
            },
        );
    }

    pub fn find(&mut self, inode: u64) -> Option<&InodeDBEntry> {
        //PLANNED: touch/refresh self.inodedb caches
        self.inode_to_identifier.get(&inode)
    }
}

