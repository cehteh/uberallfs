use crate::prelude::*;

use std::path::PathBuf;
use std::ffi::OsStr;
#[cfg(unix)] use std::os::unix::ffi::OsStrExt;

use crate::identifier::Identifier;

/// ObjectStore Path handling
pub struct OPath(PathBuf);


impl From<&[u8]> for OPath {
    fn from(s: &[u8]) -> Self {
        OPath(PathBuf::from(OsStr::from_bytes(s)))
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
}
