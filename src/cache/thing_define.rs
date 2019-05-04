extern crate r2d2;

use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use crate::*;
use crate::dao::ThingDefineDaoImpl;
use crate::models::define::ThingDefineCacheTrait;
use crate::models::define::ThingDefineDaoTrait;

lazy_static! {
    static ref CACHE: Mutex<LruCache<Thing, RawThingDefine>> = Mutex::new(LruCache::<Thing, RawThingDefine>::with_expiry_duration(Duration::from_secs(3600)));
}

pub struct ThingDefineCacheImpl;

impl ThingDefineCacheTrait for ThingDefineCacheImpl {
    fn get(&self, thing: &Thing) -> Result<RawThingDefine> {
//        debug!("get `ThingDefine` from cache for thing : {:?}", thing);
        if thing.get_full_key().is_empty() {
            return Err(NatureError::VerifyError("[biz] must not be empty!".to_string()));
        }
        let mut cache = CACHE.lock().unwrap();
        {   // An explicit scope to avoid cache.insert error
            if let Some(x) = cache.get(thing) {
                return Ok(x.clone());
            };
        };
        match ThingDefineDaoImpl::get(&thing)? {
            None => Err(NatureError::ThingNotDefined(format!("{} not defined", thing.get_full_key()))),
            Some(def) => {
                cache.insert(thing.clone(), def.clone());
                Ok(def)
            }
        }
    }
}

