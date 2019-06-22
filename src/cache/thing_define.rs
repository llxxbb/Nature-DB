extern crate r2d2;

use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use crate::*;
use crate::dao::ThingDefineDaoImpl;
use crate::models::define::ThingDefineDaoTrait;

lazy_static! {
    static ref CACHE: Mutex<LruCache<Meta, RawThingDefine>> = Mutex::new(LruCache::<Meta, RawThingDefine>::with_expiry_duration(Duration::from_secs(3600)));
}

pub struct ThingDefineCacheImpl;

impl ThingDefineCacheImpl {
    pub fn get(thing: &Meta) -> Result<RawThingDefine> {
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

#[cfg(test)]
mod test {

    #[test]
    fn can_not_get_from_cache() {
        // TODO
//        let mocks = MyMocks::new();
//        let mut instance = Instance::new("/err").unwrap();
//        let expected_instance = instance.clone();
//        mocks.s.expect(mocks.c_thing_define.get_call(check(move |t: &&Thing| **t == expected_instance.thing)).and_return(Err(NatureError::VerifyError("test error".to_string()))));
//        let testee = InstanceServiceImpl { define_cache: mocks.c_thing_define.clone() };
//        let result = testee.verify(&mut instance);
//        assert!(result.is_err());

    }

    #[test]
    fn can_get_from_cache() {
        // TODO
//        let mocks = MyMocks::new();
//        let mut instance = Instance::new("/ok").unwrap();
//        let expected_instance = instance.clone();
//        let define = RawThingDefine::default();
//        mocks.s.expect(mocks.c_thing_define.get_call(check(move |t: &&Thing| **t == expected_instance.thing)).and_return(Ok(define)));
//        let testee = InstanceServiceImpl { define_cache: mocks.c_thing_define.clone() };
//        let result = testee.verify(&mut instance);
//        assert!(result.is_ok());

    }
}
