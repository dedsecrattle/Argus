pub mod file;
pub mod storage_trait;

pub use file::FileStorage;
pub use storage_trait::{NoopStorage, Storage, url_to_fragment};

pub fn init_storage() {}
