use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use crate::{MetaCacheGetter, MetaGetter, Relation, RelationGetter, Relations};

/// all flows for one upper `Meta` and what a chance to lower `group`
type ITEM = Vec<Relation>;
type CACHE = Mutex<LruCache<String, ITEM>>;
lazy_static! {
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<String, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type RelationCacheGetter = fn(&str, RelationGetter, MetaCacheGetter, &MetaGetter) -> Relations;

pub struct RelationCacheImpl;

impl RelationCacheImpl {
    pub fn get(meta_from: &str, getter: RelationGetter, meta_cache: MetaCacheGetter, meta: &MetaGetter) -> Relations {
        let mut cache = CACHE_MAPPING.lock().unwrap();
        if let Some(rtn) = cache.get(meta_from) {
            return Ok(rtn.clone());
        }
        let rtn = getter(meta_from, meta_cache, meta)?;
        {
            let cpy = rtn.clone();
            cache.insert(meta_from.to_string(), rtn);
            Ok(cpy)
        }
    }
}

#[cfg(test)]
mod test {
    use nature_common::{Meta, NatureError, Result};

    use crate::RawMeta;

    use super::*;

    #[test]
    fn get_error() {
        let from = "B:error:1";
        let mg: &MetaGetter = &(meta as MetaGetter);

        // this will call mocker
        let result = RelationCacheImpl::get(&from, rtn_err, meta_cache, mg);
        assert_eq!(result, Err(NatureError::EnvironmentError("can't connect".to_string())));
        // error can't be catched
        let result = RelationCacheImpl::get(&from, rtn_err2, meta_cache, mg);
        assert_eq!(result, Err(NatureError::EnvironmentError("another error".to_string())));
    }

    /// test cache also
    #[test]
    fn get_none() {
        let mg: &MetaGetter = &(meta as MetaGetter);
        let from = "B:none:1";
        // this will call mocker
        let result = RelationCacheImpl::get(&from, rtn_none, meta_cache, mg);
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        assert_eq!(result.is_empty(), true);
        // and the repeated call will not call mocker but get from cache
        let result = RelationCacheImpl::get(&from, rtn_err, meta_cache, mg);
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        assert_eq!(result.is_empty(), true);
    }

    fn rtn_err(_: &str, _: MetaCacheGetter, _: &MetaGetter) -> Relations {
        Err(NatureError::EnvironmentError("can't connect".to_string()))
    }

    fn rtn_err2(_: &str, _: MetaCacheGetter, _: &MetaGetter) -> Relations {
        Err(NatureError::EnvironmentError("another error".to_string()))
    }

    fn rtn_none(_: &str, _: MetaCacheGetter, _: &MetaGetter) -> Relations {
        Ok(vec![])
    }

    fn meta_cache(meta_str: &str, _: &MetaGetter) -> Result<Meta> {
        Ok(Meta::from_string(meta_str)?)
    }

    fn meta(m: &str) -> Result<Option<RawMeta>> {
        Ok(Some(RawMeta::from(Meta::from_string(m)?)))
    }
}
