use crate::prelude::*;

use std::ffi::OsStr;
use std::fmt;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Components, Iter, PathBuf};

use crate::identifier::Identifier;

/// ObjectStore Path handling
#[derive(PartialEq)]
pub struct OPath(PathBuf);

impl From<&OsStr> for OPath {
    fn from(ostr: &OsStr) -> Self {
        OPath(PathBuf::from(ostr))
    }
}

impl From<&[u8]> for OPath {
    fn from(s: &[u8]) -> Self {
        OPath(PathBuf::from(OsStr::from_bytes(s)))
    }
}

impl fmt::Debug for OPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{:?}", self.as_os_str()))
    }
}

impl Default for OPath {
    fn default() -> Self {
        Self::new()
    }
}

impl OPath {
    pub fn new() -> Self {
        OPath(PathBuf::new())
    }

    pub fn prefix(prefix: &OsStr) -> Self {
        OPath(PathBuf::from(prefix))
    }

    pub fn push(mut self, name: &OsStr) -> Self {
        self.0.push(name);
        self
    }

    /// normalize a path by removing all current dir ('.') and parent dir ('*/..') references.
    pub fn normalize(self) -> Result<Self> {
        let mut new_path = PathBuf::new();
        for p in self.0.iter() {
            if p != "." {
                if p == ".." {
                    if !new_path.pop() {
                        bail!(ObjectStoreError::NoParent)
                    }
                } else {
                    new_path.push(p);
                }
            }
        }

        Ok(OPath(new_path))
    }

    //TODO: push subobject
    pub fn push_identifier(mut self, identifier: &Identifier) -> Self {
        let bytes = identifier.id_base64().0;
        self.0.push(OsStr::from_bytes(&bytes[..2]));
        self.0.push(OsStr::from_bytes(&bytes));
        self
    }

    pub fn push_link(mut self, identifier: &Identifier) -> Self {
        self.0.push(OsStr::from_bytes(&crate::VERSION_PREFIX));
        self.0.push(OsStr::from_bytes(&identifier.id_base64().0));
        self
    }

    pub fn set_file_name(mut self, name: &OsStr) -> Self {
        self.0.set_file_name(name);
        self
    }

    pub fn set_extension(mut self, ext: &OsStr) -> Self {
        self.0.set_extension(ext);
        self
    }

    pub fn as_os_str(&self) -> &OsStr {
        self.0.as_os_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_os_str().as_bytes()
    }

    pub fn components(&self) -> Components {
        self.0.components()
    }

    pub fn iter(&self) -> Iter {
        self.0.iter()
    }
}
