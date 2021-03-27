use crate::prelude::*;

use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};

pub struct UberallFS;

impl Filesystem for UberallFS {}
