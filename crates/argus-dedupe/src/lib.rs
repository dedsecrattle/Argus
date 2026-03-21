pub mod bloom;
pub mod hybrid;
pub mod seen;
pub mod set_trait;
pub mod simhash;

#[cfg(feature = "redis")]
pub mod redis;

pub use bloom::BloomDeduplicator;
pub use hybrid::{HybridSeenSet, HybridStats};
pub use seen::SeenUrlSet;
pub use set_trait::SeenSet;
pub use simhash::Simhash;

#[cfg(feature = "redis")]
pub use redis::RedisSeenSet;
