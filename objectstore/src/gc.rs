use std::ffi::OsStr;
use std::sync::Arc;
use std::collections::{HashSet, VecDeque};

use uberall::clap::ArgMatches;
use uberall::parking_lot::Mutex;

use crate::prelude::*;
use crate::IdentifierBin;
use crate::Identifier;
use crate::object::{DeleteMethod, Object};
use crate::objectstore::{LockingMethod::*, ObjectStore};

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
            self.collect_objects_recursive(root, &mut in_use)?;
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
                self.delete(id)
                // TODO: report expire
            } else {
                let object = Object::from(id);
                println!("Would {}: {}", object.delete_method(), object.identifier());
                // TODO: expire
                Ok(())
            }
        })
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

    /// Starting from a given root, walk all objects and store their identifiers in the given in_use HashSet.
    /// Can be called multiple times with differnt roots to fill the 'in_use' set incrementally.
    pub fn collect_objects_recursive(
        &self,
        root: &Identifier,
        in_use: &mut HashSet<IdentifierBin>,
    ) -> Result<()> {
        // stores discovered directories not yet progressed
        let to_do = Arc::new(Mutex::new(VecDeque::<Identifier>::new()));
        to_do.lock().push_back(root.clone());

        // stores all found identifiers in binary form
        let in_use = Arc::new(Mutex::new(in_use));

        while let Some(id) = {
            let v = to_do.lock().pop_front();
            v
        } {
            let id_bin = id.id_bin();
            let mut in_use1 = in_use.lock();
            if !in_use1.contains(&id_bin) {
                trace!("dir: {:?}", id);
                in_use1.insert(id_bin);
                drop(in_use1);
                for (name, entry) in self.list_directory(&id)? {
                    trace!("found: {:?}: {:?}", name, entry);
                    match entry.object_type() {
                        crate::ObjectType::File => {
                            in_use.lock().insert(entry.id_bin());
                        }
                        crate::ObjectType::Directory => {
                            let contains_not = !in_use.lock().contains(&entry.id_bin());
                            if contains_not {
                                to_do.lock().push_back(entry);
                            }
                        }
                        _ => {
                            return Err(ObjectStoreError::UnsupportedObjectType(
                                entry.kind().components(),
                            )
                            .into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
