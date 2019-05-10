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
extern crate nature_common;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use nature_common::*;

pub use self::cache::*;
pub use self::conn::*;
pub use self::dao::*;
pub use self::define::*;
pub use self::models::*;
pub use self::orm::*;
pub use self::raw_models::*;

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

pub mod service;

mod conn;