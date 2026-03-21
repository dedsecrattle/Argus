pub mod seen;
pub mod set_trait;
pub mod simhash;

#[cfg(feature = "redis")]
pub mod redis;

pub use seen::SeenUrlSet;
pub use set_trait::SeenSet;
pub use simhash::Simhash;

#[cfg(feature = "redis")]
pub use redis::RedisSeenSet;
