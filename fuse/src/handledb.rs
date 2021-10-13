use std::io;
use std::sync::Arc;

use uberall::{libc, parking_lot::Mutex};
use objectstore::Handle;

use crate::prelude::*;

type NextFree = usize;

#[derive(Debug)]
enum Entry {
    Invalid(NextFree),
    Valid(Arc<Mutex<Handle>>),
}

use Entry::*;

#[derive(Debug)]
pub struct HandleDb {
    handles:  Mutex<Vec<Entry>>,
    free_idx: NextFree, // linked list of free positions
}

/// Holds File and Directory Handles mapped to u64 indices
impl HandleDb {
    /// Create a HandleDb
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        let mut handles = Vec::with_capacity(capacity);
        handles.push(Invalid(0));

        Ok(HandleDb {
            handles:  Mutex::new(handles),
            free_idx: 0,
        })
    }

    /// Pushes a new handle onto the DB, returns an u64 representing it
    pub fn store(&mut self, handle: Handle) -> u64 {
        let mut handles = self.handles.lock();
        let handle = Valid(Arc::new(Mutex::new(handle)));
        let ret = if let Invalid(next) = handles[self.free_idx] {
            let free = self.free_idx;
            handles[free] = handle;
            self.free_idx = next;
            free
        } else {
            handles.push(handle);
            handles.len() - 1
        };
        ret as u64
    }

    /// get handle by index
    pub fn get(&mut self, fh: u64) -> Option<Arc<Mutex<Handle>>> {
        let fh = fh as usize;
        let handles = self.handles.lock();
        match handles.get(fh) {
            Some(Valid(handle)) => Some(handle.clone()),
            _ => None,
        }
    }

    /// Drops a Handle from the database
    pub fn drop(&mut self, fh: u64) -> io::Result<()> {
        let fh = fh as usize;
        let mut handles = self.handles.lock();
        if let Some(Valid(handle)) = handles.get(fh) {
            handles[fh] = Invalid(self.free_idx);
            self.free_idx = fh;
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(libc::EBADF))
        }
    }
}
