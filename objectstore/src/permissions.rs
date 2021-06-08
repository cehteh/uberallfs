use std::io;
use std::sync::Arc;
use std::{collections::HashMap, time};

use crate::{Identifier, Mutability, ObjectStore, ObjectType, SharingPolicy, UserId};

/// Defines when authenticated keys expire and will be removed.
///  * Never:: keeps the keys forever
///  * Exact:: The key will expire at the given time
///  * Idle:: The will expire when it wasn't used for 'idle_time'
enum KeyExpirePolicy {
    Never,
    Exact {
        at: time::Instant,
    },
    Idle {
        at: time::Instant,
        idle_time: time::Duration,
    },
}

#[derive(PartialEq, Eq, Hash)]
struct AuthenticatedEntry {
    uid: UserId,
    //PLANNED: pubkey: PublicKey,
}

/// stores authenticated keys
pub struct PermissionController {
    objectstore: Arc<ObjectStore>,
    authenticated: HashMap<AuthenticatedEntry, KeyExpirePolicy>,
    gc_countdown: usize,
}

impl PermissionController {
    /// Creates a new PermissionControler for an Objectstore
    pub fn new(objectstore: Arc<ObjectStore>) -> Self {
        Self {
            objectstore,
            authenticated: HashMap::new(),
            gc_countdown: 63,
        }
    }

    fn garbage_collect(&mut self) {
        self.gc_countdown -= 1;
        if self.gc_countdown == 0 {
            let now = time::Instant::now();
            self.authenticated.retain(|_, expire| {
                use KeyExpirePolicy::*;
                match *expire {
                    Never => true,
                    Exact { at } => at > now,
                    Idle { at, idle_time } => at > now,
                }
            });
            // gc every half capacity (-1), but no more frequent than every 63th insert
            self.gc_countdown = std::cmp::max(self.authenticated.capacity(), 128) / 2 - 1;
        }
    }

    //PLANNED:    fn lookup(&mut self) { update incremental expire

    //PLANNED: Keys are authenticated by requesting a challenge against a pubkey. When this
    //+ challenge succeeds then the Pubkey is stored as being authorized. This allowes for
    //+ handling all private key handling on a dedicated process outside of the vfs instance.
    //    pub fn auth_key(uid, PubKey, expire_policy) -> Result<Challenge> {
    //        unimplemented!()
    //    }
    //    pub fn add_key(Response) {
    //        self.garbage_collect();
    //        unimplemented!()
    //    }

    pub fn permission_check<'a>(
        &'a self,
        identifier: &'a Identifier,
        uid: Option<UserId>,
    ) -> PermissionCheck<'a> {
        PermissionCheck {
            controller: self,
            identifier,
            uid,
        }
    }
}

/// Temporary state for permission checking
#[must_use]
pub struct PermissionCheck<'a> {
    controller: &'a PermissionController,
    identifier: &'a Identifier,
    uid: Option<UserId>,
}

use Mutability::*;
use ObjectType::*;
use SharingPolicy::*;

impl PermissionCheck<'_> {
    pub fn read(&self) -> io::Result<()> {
        match self.identifier.components() {
            (_, Private | Anonymous, _) => Ok(()),
            (_, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    pub fn write(&self) -> io::Result<()> {
        match self.identifier.components() {
            (File, Private, _) => Ok(()),
            (File, _, Immutable) => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            (File, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    pub fn append(&self) -> io::Result<()> {
        match self.identifier.components() {
            (File, Private, _) => Ok(()),
            (File, _, Immutable) => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            (File, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    pub fn list(&self) -> io::Result<()> {
        match self.identifier.components() {
            (Directory, Private | Anonymous, _) => Ok(()),
            (Directory, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    pub fn add(&self) -> io::Result<()> {
        match self.identifier.components() {
            (Directory, Private, _) => Ok(()),
            (Directory, _, Immutable) => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            (Directory, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    pub fn rename(&self) -> io::Result<()> {
        match self.identifier.components() {
            (Directory, Private, _) => Ok(()),
            (Directory, _, Immutable) => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            (Directory, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    pub fn delete(&self) -> io::Result<()> {
        match self.identifier.components() {
            (Directory, Private, _) => Ok(()),
            (Directory, _, Immutable) => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            (Directory, PublicAcl, _) => unimplemented!(),
            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }
}
