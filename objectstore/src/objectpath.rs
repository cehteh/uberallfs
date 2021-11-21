use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use crate::prelude::*;
use crate::identifier::Identifier;

#[inline]
pub fn from_bytes(bytes: &[u8]) -> PathBuf {
    let bytes: &OsStr = OsStrExt::from_bytes(bytes);
    PathBuf::from(bytes)
}

pub trait ObjectPath {
    // TODO: see testpath/absolutize
    fn normalize(&mut self) -> Result<&mut Self>;

    fn push_identifier(&mut self, identifier: &Identifier) -> &mut Self;

    fn push_link(&mut self, identifier: &Identifier) -> &mut Self;
}

impl ObjectPath for PathBuf {
    /// normalize a path by removing all current dir ('.') and parent dir
    /// ('*/..') references.
    fn normalize(&mut self) -> Result<&mut Self> {
        let mut new_path = PathBuf::new();
        for p in self.iter() {
            if p != "." {
                if p == ".." {
                    if !new_path.pop() {
                        return Err(ObjectStoreError::NoParent.into());
                    }
                } else {
                    new_path.push(p);
                }
            }
        }

        *self = new_path;
        Ok(self)
    }

    // TODO: push subobject

    fn push_identifier(&mut self, identifier: &Identifier) -> &mut Self {
        let bytes = identifier.id_base64().0;
        self.push(OsStr::from_bytes(&bytes[..2]));
        self.push(OsStr::from_bytes(&bytes));
        self
    }

    /// create a '.uberallfs./flipbase64identifier' special link
    fn push_link(&mut self, identifier: &Identifier) -> &mut Self {
        self.push(OsStr::from_bytes(&crate::RESERVED_PREFIX));
        self.push(OsStr::from_bytes(&identifier.id_base64().0));
        self
    }
}
