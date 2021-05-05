use crate::prelude::*;

use std::io;
use std::sync::Arc;

use parking_lot::Mutex;

use objectstore::Handle;

enum Entry {
    Invalid(usize),
    Valid(Arc<Handle>),  // RefCell?
}

use Entry::*;

pub struct HandleDb {
    handles: Mutex<Vec<Entry>>,
    free_idx: usize, // linked list of free positions
}

/// Holds File and Directory Handles mapped to u64 indices
impl HandleDb {

    /// Create a HandleDb
    pub fn with_capacity(capacity: usize) -> Self {
        let mut handles = Vec::with_capacity(capacity);
        handles.push(Invalid(0));

        HandleDb {
            handles: Mutex::new(handles),
            free_idx: 0,
        }
    }

    /// Pushes a new handle onto the DB, returns an u64 representing it
    pub fn store(&mut self, handle: Handle) -> u64 {
        let mut handles = self.handles.lock();
        let ret = if let Invalid(next) = handles[self.free_idx] {
                let free = self.free_idx;
                handles[free] = Valid(Arc::new(handle));
                self.free_idx = next;
                free
            } else {
                handles.push(Valid(Arc::new(handle)));
                handles.len() - 1
        };
        ret as u64
    }

    /// get handle by index
    pub fn get(&mut self, fh: u64) -> Option<Arc<Handle>> {
        let fh = fh as usize;
        let mut handles = self.handles.lock();
        if let Some(Valid(handle)) = handles.get(fh) {
            Some(Arc::clone(handle))
        } else {
            None
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
