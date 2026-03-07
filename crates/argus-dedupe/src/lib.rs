pub mod seen;
pub mod set_trait;

#[cfg(feature = "redis")]
pub mod redis;

pub use seen::SeenUrlSet;
pub use set_trait::SeenSet;

#[cfg(feature = "redis")]
pub use redis::RedisSeenSet;
