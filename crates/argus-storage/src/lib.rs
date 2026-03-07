pub mod file;
pub mod storage_trait;

pub use file::FileStorage;
pub use storage_trait::{url_to_fragment, NoopStorage, Storage};

pub fn init_storage() {}
