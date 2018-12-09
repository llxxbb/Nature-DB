extern crate r2d2;

use ::*;
use lru_time_cache::LruCache;
use std::sync::Mutex;
use std::time::Duration;

lazy_static! {
    static ref CACHE: Mutex<LruCache<Thing, ThingDefine>> = Mutex::new(LruCache::<Thing, ThingDefine>::with_expiry_duration(Duration::from_secs(3600)));
}

pub struct ThingDefineCacheImpl;

impl ThingDefineCacheTrait for ThingDefineCacheImpl {
    fn get(&self, thing: &Thing) -> Result<ThingDefine> {
//        debug!("get `ThingDefine` from cache for thing : {:?}", thing);
        if thing.key.is_empty() {
            return Err(NatureError::VerifyError("[biz] must not be empty!".to_string()));
        }
        let mut cache = CACHE.lock().unwrap();
        {   // An explicit scope to avoid cache.insert error
            if let Some(x) = cache.get(thing) {
                return Ok(x.clone());
            };
        };
        match ThingDefineDaoImpl::get(&thing)? {
            None => Err(NatureError::ThingNotDefined(format!("{} not defined", thing.key))),
            Some(def) => {
                cache.insert(thing.clone(), def.clone());
                Ok(def)
            }
        }
    }
}

