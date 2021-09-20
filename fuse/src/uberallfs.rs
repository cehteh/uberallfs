use crate::prelude::*;

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uberall::libc;
use uberall::ipc_channel::ipc;

use objectstore::{Identifier, ObjectType, VirtualFileSystem};

use crate::{HandleDb, InodeDb};

use fuser::{
    FileAttr, FileType, Filesystem, KernelConfig, MountOption, ReplyAttr, ReplyData,
    ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen, Request,
};

type CallbackTx = ipc::IpcSender<std::option::Option<i32>>;

struct CallBack {
    callback: Option<Box<dyn FnOnce(CallbackTx, Option<i32>)>>,
    tx: Option<CallbackTx>,
}

impl CallBack {
    fn new() -> Self {
        CallBack {
            callback: None,
            tx: None,
        }
    }

    pub fn set(
        &mut self,
        callback: Box<dyn FnOnce(CallbackTx, Option<i32>)>,
        tx: Option<CallbackTx>,
    ) -> &Self {
        self.callback = Some(callback);
        self.tx = tx;
        self
    }

    pub fn callback_once(&mut self, error: Option<i32>) {
        if let Some(callback) = self.callback.take() {
            trace!("callback");
            callback(self.tx.take().unwrap(), error);
        } else {
            trace!("no callback");
        }
    }
}

pub struct UberallFS {
    vfs: VirtualFileSystem,
    inodedb: InodeDb,
    handledb: HandleDb,
    callback: CallBack,
}

impl fmt::Debug for UberallFS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UberallFS")
            .field("vfs", &self.vfs)
            .field("inodedb", &self.inodedb)
            .field("handledb", &self.handledb)
            .field("callback.is_some()", &self.callback.callback.is_some())
            .finish()
    }
}

impl UberallFS {
    pub fn new(objectstore_dir: &Path) -> Result<UberallFS> {
        Ok(UberallFS {
            vfs: VirtualFileSystem::new(objectstore_dir)?,
            inodedb: InodeDb::new()?,
            handledb: HandleDb::with_capacity(1024)?,
            callback: CallBack::new(),
        })
    }

    pub fn with_callback<C: FnOnce(CallbackTx, Option<i32>) + Copy + 'static>(
        mut self,
        callback: C,
        tx: Option<CallbackTx>,
    ) -> Self {
        self.callback.set(Box::new(callback), tx);
        self
    }

    pub fn callback_once(&mut self, error: Option<i32>) {
        self.callback.callback_once(error);
    }

    pub fn mount(
        mut self,
        mountpoint: &Path,
        _offline_todo: bool,
        root: &OsStr,
        _options_planned: Option<Vec<String>>,
    ) -> Result<()> {
        let mut options = vec![
            MountOption::RO,
            MountOption::FSName("uberallfs".to_string()),
        ];
        options.push(MountOption::AutoUnmount); //TODO: optarg?

        let identifier = self.vfs.path_lookup(0, Path::new(root))?;

        self.inodedb.store(1, identifier);
        //FIXME: for the real metadata/ino, make '1' a special case UberallFS::root_ino
        fuser::mount2(&mut self, mountpoint, &options)
            .or_else(|err| {
                error!("mounting filesystem: {:?}", err);
                self.callback_once(err.raw_os_error());
                Err(err)
            })
            .or(Ok(()))
    }
}

impl Filesystem for &mut UberallFS {
    //PLANNED: investigate what to do async

    fn init(
        &mut self,
        _req: &Request<'_>,
        _config: &mut KernelConfig,
    ) -> std::result::Result<(), libc::c_int> {
        trace!("init filesystem");
        self.callback.callback_once(None);
        Ok(())
    }

    fn access(&mut self, req: &Request<'_>, ino: u64, mode: i32, reply: ReplyEmpty) {
        if let Some(entry) = self.inodedb.get(ino) {
            if let Ok(()) = self.vfs.access(req.uid(), entry.as_identifier(), mode) {
                trace!("access ok {}", ino);
                return reply.ok();
            } else {
                warn!("access error {} {}", ino, std::io::Error::last_os_error());
                return reply.error(std::io::Error::last_os_error().raw_os_error().unwrap_or(0));
            }
        }
        error!("inode not found {}", ino);
        reply.error(libc::ENOENT);
    }

    fn lookup(&mut self, req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if let Some(entry) = self.inodedb.get(parent) {
            if let Ok(sub_id) = self.vfs.sub_lookup(req.uid(), entry.as_identifier(), name) {
                trace!("sub_id: {:?}", sub_id);
                if let Ok(metadata) = self.vfs.metadata(req.uid(), &sub_id) {
                    let entry = self.inodedb.store(metadata.stat().st_ino, sub_id);
                    let sub_id = entry.as_identifier();
                    return reply.entry(
                        &Duration::from_secs(600),
                        &stat_to_fileattr(metadata.stat(), identifier_to_filetype(sub_id)),
                        0, //TODO: generation
                    );
                }
            }
        }
        reply.error(libc::ENOENT);
    }

    // fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
    //     if let Some(entry) = self.inodedb.get(ino) {
    //         trace!("id: {:?}", entry.as_identifier());
    //         if let Ok(metadata) = self.vfs.object_metadata(&entry.as_identifier()) {
    //             return reply.attr(
    //                 &Duration::from_secs(600),
    //                 &stat_to_fileattr(
    //                     metadata.stat(),
    //                     identifier_to_filetype(&entry.as_identifier()),
    //                 ),
    //             );
    //         }
    //     }
    //     reply.error(libc::ENOENT);
    // }

    // fn opendir(&mut self, _req: &Request<'_>, ino: u64, _flags: i32, reply: ReplyOpen) {
    //     match self.inodedb.get(ino) {
    //         Some(directory) if directory.as_identifier().object_type() == ObjectType::Directory => {
    //             match self.vfs.list_directory(directory.as_identifier()) {
    //                 Ok(handle) => {
    //                     reply.opened(
    //                         self.handledb.store(handle),
    //                         0, //TODO: what flags?
    //                     )
    //                 }
    //                 Err(err) => {
    //                     reply.error(err.raw_os_error().unwrap_or(libc::ENOENT));
    //                 }
    //             }
    //         }
    //         Some(_) => {
    //             reply.error(libc::ENOTDIR);
    //         }
    //         None => {
    //             reply.error(libc::ENOENT);
    //         }
    //     }
    // }

    // fn releasedir(
    //     &mut self,
    //     _req: &Request<'_>,
    //     _ino: u64,
    //     fh: u64,
    //     _flags: i32,
    //     reply: ReplyEmpty,
    // ) {
    //     if let Err(err) = self.handledb.drop(fh) {
    //         reply.error(err.raw_os_error().unwrap());
    //     } else {
    //         reply.ok();
    //     }
    // }

    // fn readdir(
    //     &mut self,
    //     _req: &Request<'_>,
    //     _ino: u64,
    //     fh: u64,
    //     _offset: i64,
    //     mut reply: ReplyDirectory,
    // ) {
    //     use std::ops::Deref;
    //     use std::ops::DerefMut;
    //     use std::os::unix::io::AsRawFd;
    //     trace!("readdir: {} {} {}", _ino, fh, _offset);
    //
    //     //TODO: from vfs
    //     /*
    //     let entry = self.handledb.get(fh);
    //     if let Some(handle) = entry.as_deref() {
    //         if let Handle::DirIter(dir_iter) = handle.lock().deref_mut() {
    //             for entry in dir_iter {
    //                 trace!("iter: {:?} ", entry);
    //
    //                 match entry {
    //                     Ok(entry) => {
    //                         //TODO:
    //                         entry.file_name();
    //
    //                         let ino = _ino+1;
    //                         let offset = -1;
    //                         reply.add(
    //                             ino,
    //                             offset,
    //                             file_type(&entry),
    //                             entry.file_name(),
    //                         );
    //                     }
    //                     Err(err) => {
    //                         return reply.error(err.raw_os_error().unwrap())
    //                     }
    //                 }
    //             }
    //             reply.ok()
    //         } else {
    //             reply.error(libc::ENOTDIR)
    //         }
    //     } else {
    //         reply.error(libc::ENOENT);
    //     }
    //      */
    // }

    /*
    //TODO:
        pub fn init(
        pub fn destroy(&mut self, _req: &Request<'_>) { ... }
        pub fn forget(&mut self, _req: &Request<'_>, _ino: u64, _nlookup: u64) { ... }
        pub fn getattr(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyAttr) { ... }
        pub fn setattr(
        pub fn readlink(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyData) { ... }
        pub fn mknod(
        pub fn mkdir(
        pub fn unlink(
        pub fn rmdir(
        pub fn symlink(
        pub fn rename(
        pub fn link(
        pub fn open(
        pub fn read(
        pub fn write(
        pub fn flush(
        pub fn release(
        pub fn fsync(
        pub fn readdirplus(
        pub fn fsyncdir(
        pub fn statfs(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyStatfs) { ... }
        pub fn setxattr(
        pub fn getxattr(
        pub fn listxattr(
        pub fn removexattr(
        pub fn create(
        pub fn getlk(
        pub fn setlk(
        pub fn bmap(
        pub fn ioctl(
        pub fn fallocate(
        pub fn lseek(
        pub fn copy_file_range(
     */
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
        flags: 0,
    }
}
