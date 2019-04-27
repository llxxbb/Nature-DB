//! Define the data used all over the project, not only by `fg-service`

#![allow(proc_macro_derive_resolution_fallback)]
#![feature(custom_attribute)]
#![feature(rustc_private)]

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
pub use self::conn::*;
pub use self::orm::*;

pub mod schema;

#[cfg(feature = "sqlite")]
mod sqlite;
mod cache;
mod orm;
mod dao;
mod raw_models;
mod models;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(test)]
mod test_util;

pub mod service;

mod conn {
    #[cfg(feature = "mysql")]
    pub use crate::mysql::CONN;
    #[cfg(feature = "sqlite")]
    pub use crate::sqlite::CONN;
}