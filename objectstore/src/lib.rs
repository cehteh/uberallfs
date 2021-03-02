mod args;
pub use self::args::args;

struct ObjectStore {
}

impl ObjectStore {
    //create
    //open

}

impl Drop for ObjectStore {
    fn drop(&mut self){
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
