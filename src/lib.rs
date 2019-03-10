//! Define the data used all over the project, not only by `fg-service`

#![allow(proc_macro_derive_resolution_fallback)]
#![feature(custom_attribute)]
#![feature(rustc_private)]
#![feature(range_contains)]

extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
#[cfg(test)]
extern crate mockers;
#[cfg(test)]
extern crate mockers_derive;
extern crate nature_common;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use nature_common::*;

pub use self::cache::*;
pub use self::converter_cfg::*;
pub use self::define::*;
pub use self::instance::*;
pub use self::orm::*;
pub use self::plan::*;
pub use self::sqlite::*;
pub use self::task_type::*;
pub use self::thing_define::*;

mod thing_define;
mod task_type;
mod converter_cfg;

mod sqlite;
mod cache;
mod define;
mod orm;
mod instance;
mod plan;

pub mod service;