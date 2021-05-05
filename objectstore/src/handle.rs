use crate::prelude::*;

pub enum Handle {
    Dir(openat::Dir),
    DirIter(openat::DirIter),
    File(std::fs::File),
}

// impl Handle
// change_access() etc
