use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use crate::{MetaCache, MetaDao, Relation, RelationDao, Relations};

use super::RUNTIME;

/// all flows for one upper `Meta` and what a chance to lower `group`
type ITEM = Vec<Relation>;
type CACHE = Mutex<LruCache<String, ITEM>>;
lazy_static! {
    pub static ref C_R: RelationCacheImpl = RelationCacheImpl {};
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<String, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}

pub trait RelationCache {
    fn get<R, MC, M>(meta_from: &str, getter: R, meta_cache: MC, meta: M) -> Relations
        where R: RelationDao, MC: MetaCache + Copy, M: MetaDao + Copy;
}

pub struct RelationCacheImpl;

impl RelationCache for RelationCacheImpl {
    fn get<R, MC, M>(meta_from: &str, getter: R, meta_cache: MC, meta: M) -> Relations
        where R: RelationDao, MC: MetaCache + Copy, M: MetaDao + Copy {
        let mut cache = CACHE_MAPPING.lock().unwrap();
        if let Some(rtn) = cache.get(meta_from) {
            return Ok(rtn.clone());
        }
        let rtn = RUNTIME.unwrap().block_on(getter.get_relations(meta_from, meta_cache, meta));
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
        let mg = MetaMock {};
        let from = "B:error:1";

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
        let mg = MetaMock {};
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

    fn rtn_err<MC, M>(_: &str, _: MC, _: M) -> Relations
        where MC: MetaCache, M: MetaDao
    {
        Err(NatureError::EnvironmentError("can't connect".to_string()))
    }

    fn rtn_err2<MC, M>(_: &str, _: MC, _: M) -> Relations
        where MC: MetaCache, M: MetaDao
    {
        Err(NatureError::EnvironmentError("another error".to_string()))
    }

    fn rtn_none<MC, M>(_: &str, _: MC, _: M) -> Relations
        where MC: MetaCache, M: MetaDao {
        Ok(vec![])
    }

    fn meta_cache<M>(meta_str: &str, _: M) -> Result<Meta>
        where M: MetaDao
    {
        Ok(Meta::from_string(meta_str)?)
    }

    struct MetaMock;

    #[async_trait]
    impl MetaDao for MetaMock {
        async fn get(&self, m: &str) -> Result<Option<RawMeta>> {
            Ok(Some(RawMeta::from(Meta::from_string(m)?)))
        }

        async fn insert(&self, define: &RawMeta) -> Result<usize> {
            unimplemented!()
        }

        async fn update_flag(&self, meta_str: &str, flag_f: i32) -> Result<usize> {
            unimplemented!()
        }

        async fn delete(&self, m: &Meta) -> Result<usize> {
            unimplemented!()
        }
    }
}
