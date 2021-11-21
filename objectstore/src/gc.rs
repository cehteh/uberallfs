use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use uberall::clap::ArgMatches;

use crate::{identifier, prelude::*};
use crate::identifier_kind::*;
use crate::IdentifierBin;
use crate::Identifier;
use crate::object::{DeleteMethod, Object};
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

    /// Run garbage collection on the objectstore.  Garbage collection discovers all
    /// referenced objects starting from the given roots and then removes all objects that are
    /// not referenced. The 'dry_run' parameter make it only report what would been done on
    /// stdout without changing anything.
    pub fn gc(&self, roots: &[Identifier], dry_run: bool) -> Result<()> {
        self.unreachable(roots)?.try_for_each(|id| {
            if !dry_run {
                self.delete(id).into()
                // TODO: report expire
            } else {
                let object = Object::from(id);
                println!("Would {}: {}", object.delete_method(), object.identifier());
                // TODO: expire
                Ok(())
            }
        });
        Ok(())
    }

    /// Delete an object from the objectstore. This is the low-level object deletion which
    /// will remove the object data no matter if they are still in use. Distributed objects
    /// will be put into the 'delete' directory from where they will be expired later.
    pub(crate) fn delete(&self, id: Identifier) -> Result<()> {
        let object = Object::from(id);
        let delete_method = object.delete_method();
        trace!("{}: {}", delete_method, object.identifier());
        match delete_method {
            DeleteMethod::Immediate => Ok(self
                .objects
                .remove_recursive_atomic(&object.identifier().to_pathbuf(), "tmp")?),
            DeleteMethod::Expire => Ok(self
                .objects
                .local_rename(&object.identifier().to_pathbuf(), "delete")?),
            DeleteMethod::Unknown => Err(ObjectStoreError::UnsupportedObjectType(
                object.identifier().components(),
            )
            .into()),
        }
    }
}
