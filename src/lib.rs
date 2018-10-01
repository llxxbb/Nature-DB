//! Define the data used all over the project, not only by `fg-service`

#![allow(proc_macro_derive_resolution_fallback)]
#![feature(custom_attribute)]
#![feature(rustc_private)]
#![feature(int_to_from_bytes)]
#![feature(range_contains)]
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
extern crate nature_common;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use nature_common::*;
pub use self::cache::*;
pub use self::converter_cfg::*;
pub use self::delivery::*;
pub use self::instance::*;
pub use self::orm::*;
pub use self::plan::*;
pub use self::sqlite::*;
#[cfg(test)]
pub use self::test::*;
pub use self::thing::*;
pub use self::define::*;

mod thing;
mod delivery;
#[cfg(test)]
mod test;
mod converter_cfg;

mod sqlite;
mod cache;
mod define;
mod orm;
mod instance;
mod plan;