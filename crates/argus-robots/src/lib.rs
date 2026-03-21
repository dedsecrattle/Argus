mod parser;
mod cache;

pub use cache::RobotsCache;
pub use parser::{RobotsTxt, Rule};

pub fn is_allowed(_url: &str) -> bool {
    true
}
