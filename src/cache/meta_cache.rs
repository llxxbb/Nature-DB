extern crate r2d2;

use std::convert::TryInto;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::{Meta, MetaType, NatureError, Result};

use crate::MetaGetter;

lazy_static! {
    static ref CACHE: Mutex<LruCache<String, Meta>> = Mutex::new(LruCache::<String, Meta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type MetaCacheGetter = fn(&str, MetaGetter) -> Result<Meta>;

pub struct MetaCacheImpl;

impl MetaCacheImpl {
    pub fn get(meta_str: &str, getter: MetaGetter) -> Result<Meta> {
        if meta_str.is_empty() {
            let error = NatureError::VerifyError("[biz] can not be empty!".to_string());
            warn!("{}", error);
            return Err(error);
        }
        {   // An explicit scope to avoid cache.insert error
            let mut cache = CACHE.lock().unwrap();
            if let Some(x) = cache.get(meta_str) {
                return Ok(x.clone());
            };
        };
        match getter(meta_str)? {
            None => {
                let m = Meta::from_string(meta_str)?;
                match m.get_meta_type() {
                    MetaType::Null => {
                        let mut cache = CACHE.lock().unwrap();
                        cache.insert(meta_str.to_string(), m.clone());
                        Ok(m)
                    }
                    MetaType::Dynamic => {
                        let mut cache = CACHE.lock().unwrap();
                        cache.insert(meta_str.to_string(), m.clone());
                        Ok(m)
                    }
                    _ => {
                        let error = NatureError::VerifyError(format!("{} not defined", meta_str));
                        warn!("{}", error);
                        Err(error)
                    }
                }
            }
            Some(def) => {
                let meta: Meta = def.try_into()?;
                let _ = Self::check_master(&meta, getter)?;
                let mut cache = CACHE.lock().unwrap();
                cache.insert(meta_str.to_string(), meta.clone());
                Ok(meta)
            }
        }
    }

    fn check_master(meta: &Meta, getter: MetaGetter) -> Result<()> {
        match meta.get_setting() {
            None => Ok(()),
            Some(setting) => match setting.master {
                None => Ok(()),
                Some(master) => {
                    let _ = Self::get(&master, getter)?;
                    Ok(())
                }
            },
        }
    }
}

