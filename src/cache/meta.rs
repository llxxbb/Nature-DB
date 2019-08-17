extern crate r2d2;

use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use crate::*;
use crate::dao::MetaDaoImpl;

lazy_static! {
    static ref CACHE: Mutex<LruCache<Meta, RawMeta>> = Mutex::new(LruCache::<Meta, RawMeta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub struct MetaCacheImpl;

impl MetaCacheImpl {
    pub fn get(meta: &Meta) -> Result<RawMeta> {
//        debug!("get `Meta` from cache for meta : {:?}", meta);
        if meta.get_full_key().is_empty() {
            return Err(NatureError::VerifyError("[biz] must not be empty!".to_string()));
        }
        let mut cache = CACHE.lock().unwrap();
        {   // An explicit scope to avoid cache.insert error
            if let Some(x) = cache.get(meta) {
                return Ok(x.clone());
            };
        };
        match MetaDaoImpl::get(&meta)? {
            None => Err(NatureError::MetaNotDefined(format!("{} not defined", meta.get_full_key()))),
            Some(def) => {
                cache.insert(meta.clone(), def.clone());
                Ok(def)
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn can_not_get_from_cache() {
        // TODO
//        let mocks = MyMocks::new();
//        let mut instance = Instance::new("/err").unwrap();
//        let expected_instance = instance.clone();
//        mocks.s.expect(mocks.c_meta.get_call(check(move |t: &&Meta| **t == expected_instance.meta)).and_return(Err(NatureError::VerifyError("test error".to_string()))));
//        let testee = InstanceServiceImpl { define_cache: mocks.c_meta.clone() };
//        let result = testee.verify(&mut instance);
//        assert!(result.is_err());

    }

    #[test]
    fn can_get_from_cache() {
        // TODO
//        let mocks = MyMocks::new();
//        let mut instance = Instance::new("/ok").unwrap();
//        let expected_instance = instance.clone();
//        let define = RawMeta::default();
//        mocks.s.expect(mocks.c_meta.get_call(check(move |t: &&Meta| **t == expected_instance.meta)).and_return(Ok(define)));
//        let testee = InstanceServiceImpl { define_cache: mocks.c_meta.clone() };
//        let result = testee.verify(&mut instance);
//        assert!(result.is_ok());

    }
}
