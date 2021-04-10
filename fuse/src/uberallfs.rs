use crate::prelude::*;

use std::collections::hash_map::HashMap;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use objectstore::identifier::Identifier;
use objectstore::identifier_kind::ObjectType;
use objectstore::objectstore::{OPath, ObjectStore, SubObject};

use crate::inodedb::InodeDB;

use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, Request,
};

pub struct UberallFS {
    objectstore: ObjectStore,
    inodedb: InodeDB,
}

impl UberallFS {
    pub fn new(objectstore_dir: &Path) -> Result<UberallFS> {
        Ok(UberallFS {
            objectstore: ObjectStore::open(objectstore_dir)?,
            inodedb: InodeDB::new()?,
        })
    }

    pub fn mount(
        mut self,
        mountpoint: &Path,
        offline: bool,
        root: Option<&OsStr>,
        _options_planned: Option<Vec<String>>,
    ) -> Result<()> {
        let mut options = vec![
            MountOption::RO,
            MountOption::FSName("uberallfs".to_string()),
        ];
        options.push(MountOption::AutoUnmount); //TODO: optarg?

        let (identifier, none) = self.objectstore.path_lookup(root.map(From::from), None)?;
        assert_eq!(none, None);

        self.inodedb.store(1, &identifier);
        //FIXME: for the real metadata/ino, make '1' a special case UberallFS::root_ino
        fuser::mount2(self, mountpoint, &options)?;
        Ok(())
    }
}

impl Filesystem for UberallFS {
    fn access(&mut self, _req: &Request<'_>, ino: u64, mode: i32, reply: ReplyEmpty) {
        //PLANNED: store permissions in inodedb, do access check against that
        //PLANNED: check what the benefits of access() are, can we go without?

        if let Some(entry) = self.inodedb.find(ino) {
            match unsafe {
                libc::faccessat(
                    self.objectstore.get_objects_fd(),
                    entry.to_opath().into(),
                    mode,
                    0,
                )
            } {
                0 => {
                    trace!("access ok {}", ino);
                    return reply.ok();
                }
                _ => {
                    warn!("access error {} {}", ino, std::io::Error::last_os_error());
                    return reply
                        .error(std::io::Error::last_os_error().raw_os_error().unwrap_or(0));
                }
            };
        }
        error!("inode not found {}", ino);
        reply.error(libc::ENOENT);
    }

    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if let Some(entry) = self.inodedb.find(parent) {
            if let Ok(sub_id) = self
                .objectstore
                .sub_object_id(&SubObject(&entry.as_identifier(), name))
            {
                trace!("sub_id: {:?}", sub_id);
                if let Ok(metadata) = self.objectstore.object_metadata(&sub_id) {
                    self.inodedb.store(metadata.stat().st_ino, &sub_id);
                    return reply.entry(
                        &Duration::from_secs(600),
                        &stat_to_fileattr(metadata.stat(), identifier_to_filetype(&sub_id)),
                        0, //TODO: generation
                    );
                }
            }
        }

        reply.error(libc::ENOENT);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        if let Some(entry) = self.inodedb.find(ino) {
            trace!("id: {:?}", entry.as_identifier());
            if let Ok(metadata) = self.objectstore.object_metadata(&entry.as_identifier()) {
                return reply.attr(
                    &Duration::from_secs(600),
                    &stat_to_fileattr(metadata.stat(), identifier_to_filetype(&entry.as_identifier())),
                );
            }
        }
        reply.error(libc::ENOENT);
    }
}

fn unix_to_system_time(sec: libc::time_t, ns: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(sec as u64) + Duration::from_nanos(ns as u64)
}

fn identifier_to_filetype(identifier: &Identifier) -> FileType {
    match identifier.object_type() {
        ObjectType::File => FileType::RegularFile,
        ObjectType::Directory => FileType::Directory,
        _ => unimplemented!(),
    }
}

fn stat_to_fileattr(stat: &libc::stat, kind: FileType) -> FileAttr {
    FileAttr {
        ino: stat.st_ino,
        size: stat.st_size as u64,
        blocks: stat.st_blocks as u64,
        atime: unix_to_system_time(stat.st_atime, stat.st_atime_nsec),
        mtime: unix_to_system_time(stat.st_mtime, stat.st_mtime_nsec),
        ctime: unix_to_system_time(stat.st_ctime, stat.st_ctime_nsec),
        crtime: UNIX_EPOCH, //unused
        kind,
        perm: stat.st_mode as u16, //FIXME: not sure
        nlink: stat.st_nlink as u32,
        uid: stat.st_uid,
        gid: stat.st_gid,
        rdev: stat.st_rdev as u32,
        blksize: stat.st_blksize as u32,
        padding: 0,
        flags: 0,
    }
}
