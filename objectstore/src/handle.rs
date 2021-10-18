use openat_ct as openat;

#[derive(Debug)]
pub enum Handle {
    Dir(openat::Dir),
    DirIter(openat::DirIter),
    File(std::fs::File),
}

// impl Handle
// change_access() etc
