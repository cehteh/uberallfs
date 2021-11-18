use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use uberall::clap::ArgMatches;

use crate::prelude::*;
use crate::identifier_kind::*;
use crate::IdentifierBin;
use crate::Identifier;
use crate::object::Object;
use crate::objectstore::{LockingMethod::*, ObjectStore, SubObject};

pub(crate) fn opt_gc(dir: &OsStr, matches: &ArgMatches) -> Result<()> {
    let objectstore = ObjectStore::open(dir.as_ref(), WaitForLock)?;

    // PLANNED: gc may background (defaults to foreground, does this need any change in the dameonizer?)
    // PLANNED: additional roots

    let root = objectstore.get_root_id()?;
    info!("root is: {:?}", root);

    objectstore.gc(&[root], matches.is_present("dry-run"))
}

impl ObjectStore {
    /// Run garbage collection on the objectstore.  Garbage collection discovers all
    /// referenced objects starting from the given roots and then removes all objects that are
    /// not referenced. The 'dry_run' parameter make it only report what would been done on
    /// stdout without changing anything.
    pub fn gc(&self, roots: &[Identifier], dry_run: bool) -> Result<()> {
        self.unreachable(roots)?.for_each(|id| {
            // TODO: figure delete method out (immediate/expire)
            if !dry_run {
            } else {
                println!("delete: {}", id);
            }
        });
        Ok(())
    }

    /// Returns an iterator of all objects not reachable from the given roots.
    pub fn unreachable(
        &self,
        roots: &[Identifier],
    ) -> Result<impl Iterator<Item = Identifier> + '_> {
        // discover referenced objects from all roots
        let mut in_use = HashSet::<IdentifierBin>::new();
        for root in roots {
            self.collect_objects_recursive(&root, &mut in_use)?;
        }

        // iterate over all objects, filter referenced objects
        Ok(self
            .all_objects()
            .filter(move |id| !in_use.contains(&id.id_bin())))
    }
}
