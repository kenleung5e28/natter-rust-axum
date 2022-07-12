pub mod space;
pub mod user;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref USER_REGEX: Regex = Regex::new("[a-zA-Z][a-zA-Z0-9]{1,29}").unwrap();
}
