pub mod file;
pub mod s3;
pub mod storage_trait;

pub use file::FileStorage;
pub use s3::S3Storage;
pub use storage_trait::{url_to_fragment, NoopStorage, Storage};

pub fn init_storage() {}
