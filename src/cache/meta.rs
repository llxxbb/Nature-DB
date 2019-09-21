extern crate r2d2;

use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::{Meta, NatureError, Result};

use crate::{MetaGetter, RawMeta};

lazy_static! {
    static ref CACHE: Mutex<LruCache<Meta, RawMeta>> = Mutex::new(LruCache::<Meta, RawMeta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub struct MetaCacheImpl;

impl MetaCacheImpl {
    pub fn get(meta: &Meta, getter: MetaGetter) -> Result<RawMeta> {
//        debug!("get `Meta` from cache for meta : {:?}", meta);
        if meta.get_full_key().is_empty() {
            let error = NatureError::VerifyError("[biz] must not be empty!".to_string());
            warn!("{}", error);
            return Err(error);
        }
        let mut cache = CACHE.lock().unwrap();
        {   // An explicit scope to avoid cache.insert error
            if let Some(x) = cache.get(meta) {
                return Ok(x.clone());
            };
        };
        match getter(&meta)? {
            None => {
                let error = NatureError::MetaNotDefined(format!("{} not defined", meta.get_full_key()));
                warn!("{}", error);
                Err(error)
            }
            Some(def) => {
                cache.insert(meta.clone(), def.clone());
                Ok(def)
            }
        }
    }
}
