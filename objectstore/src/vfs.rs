use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::sync::Arc;

use openat_ct as openat;
use openat::Metadata;
use uberall::libc;

use crate::prelude::*;
use crate::{
    Identifier, LockingMethod::*, ObjectStore, PermissionCheck, PermissionController, SubObject,
    UserId,
};

/// Filesystem alike access layer to the objectstore. Does access checks based
/// on numeric user ids. These user ids are authenticated and mapped to pubkeys
/// in the PermissionController.
#[cfg(unix)]
#[derive(Debug)]
pub struct VirtualFileSystem {
    objectstore:           Arc<ObjectStore>,
    permission_controller: PermissionController,
}

#[cfg(unix)]
impl VirtualFileSystem {
    pub fn new(dir: &Path) -> Result<VirtualFileSystem> {
        let objectstore = Arc::new(ObjectStore::open(dir, WaitForLock)?);
        let permission_controller = PermissionController::new(objectstore.clone());
        Ok(Self {
            objectstore,
            permission_controller,
        })
    }

    /// Request a permission check on an object.
    #[inline]
    fn permission_check<'a>(
        &'a self,
        identifier: &'a Identifier,
        uid: Option<UserId>,
    ) -> PermissionCheck {
        self.permission_controller.permission_check(identifier, uid)
    }

    /// resolves the given path to an identifier.
    pub fn path_lookup(&self, uid: UserId, path: &Path) -> Result<Identifier> {
        let identifier = self.objectstore.path_lookup(path, None).map(|t| t.0)?;
        self.permission_check(&identifier, Some(uid)).list()?;
        Ok(identifier)
    }

    /// the vfs layer does access checks only against the authenticated user id.
    /// There is no concept of real or effective uid's and no groups.
    #[inline]
    pub fn access(
        &self,
        _uid: UserId,
        _identifier: &Identifier,
        _mode: libc::c_int,
    ) -> io::Result<()> {
        // dispatch on object type
        //  dispatch on mode
        // FIXME: self.permission_check(&identifier, Some(uid)).list()?;
        Ok(())
    }

    #[inline]
    pub fn sub_lookup(
        &self,
        uid: UserId,
        identifier: &Identifier,
        name: &OsStr,
    ) -> Result<Identifier> {
        // TODO: permission checks against keys
        let sub_identifier = self
            .objectstore
            .sub_object_id(&SubObject(identifier, name))?;

        self.permission_check(&sub_identifier, Some(uid)).read()?;

        Ok(sub_identifier)
    }

    #[inline]
    pub fn metadata(&self, _uid: UserId, identifier: &Identifier) -> io::Result<Metadata> {
        // TODO: permission checks against keys
        Ok(self.objectstore.object_metadata(identifier)?)
    }
}
