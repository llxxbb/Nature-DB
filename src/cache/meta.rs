extern crate r2d2;

use std::convert::TryInto;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::{Meta, NatureError, Result};

use crate::MetaGetter;

lazy_static! {
    static ref CACHE: Mutex<LruCache<String, Meta>> = Mutex::new(LruCache::<String, Meta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type MetaCacheGetter = fn(&mut Meta, MetaGetter) -> Result<()>;

pub struct MetaCacheImpl;

impl MetaCacheImpl {
    pub fn get(meta: &mut Meta, getter: MetaGetter) -> Result<()> {
//        debug!("get `Meta` from cache for meta : {:?}", meta);
        if meta.get_full_key().is_empty() {
            let error = NatureError::VerifyError("[biz] must not be empty!".to_string());
            warn!("{}", error);
            return Err(error);
        }
        let mut cache = CACHE.lock().unwrap();
        let key = meta.meta_string();
        {   // An explicit scope to avoid cache.insert error
            if let Some(x) = cache.get(&key) {
                *meta = x.clone();
                return Ok(());
            };
        };
        match getter(&meta)? {
            None => {
                let error = NatureError::MetaNotDefined(format!("{} not defined", meta.get_full_key()));
                warn!("{}", error);
                Err(error)
            }
            Some(def) => {
                cache.insert(key, def.try_into()?);
                Ok(())
            }
        }
    }
}
