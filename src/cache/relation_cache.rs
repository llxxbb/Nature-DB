use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use crate::{MetaCache, MetaDao, Relation, RelationDao, Relations};

/// all flows for one upper `Meta` and what a chance to lower `group`
type ITEM = Vec<Relation>;
type CACHE = Mutex<LruCache<String, ITEM>>;
lazy_static! {
    pub static ref C_R: RelationCacheImpl = RelationCacheImpl {};
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<String, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}

#[async_trait]
pub trait RelationCache {
    async fn get<R, MC, M>(&self, meta_from: &str, getter: &R, meta_cache: &MC, meta: &M) -> Relations
        where R: RelationDao, MC: MetaCache, M: MetaDao;
}

pub struct RelationCacheImpl;

#[async_trait]
impl RelationCache for RelationCacheImpl {
    async fn get<R, MC, M>(&self, meta_from: &str, getter: &R, meta_cache: &MC, meta: &M) -> Relations
        where R: RelationDao, MC: MetaCache, M: MetaDao {
        {
            let mut cache = CACHE_MAPPING.lock().unwrap();
            if let Some(rtn) = cache.get(meta_from) {
                return Ok(rtn.clone());
            }
        }

        let rtn = getter.get_relations(meta_from, meta_cache, meta).await?;
        {
            let cpy = rtn.clone();
            let mut cache = CACHE_MAPPING.lock().unwrap();
            cache.insert(meta_from.to_string(), rtn);
            Ok(cpy)
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::runtime::Runtime;

    use nature_common::{Meta, NatureError, Result};

    use crate::{RawMeta, RawRelation};

    use super::*;

    #[test]
    fn get_error() {
        let from = "B:error:1";

        // this will call mocker
        let mut rt = Runtime::new().unwrap();
        let result = rt.block_on(C_R.get(&from, &RMockERR {}, &MCMock {}, &MetaMock {}));
        assert_eq!(result, Err(NatureError::EnvironmentError("can't connect".to_string())));
        // error can't be catched
        let result = rt.block_on(C_R.get(&from, &RMockERR2, &MCMock {}, &MetaMock {}));
        assert_eq!(result, Err(NatureError::EnvironmentError("another error".to_string())));
    }

    /// test cache also
    #[test]
    fn get_none() {
        let from = "B:none:1";
        // this will call mocker
        let mut rt = Runtime::new().unwrap();
        let result = rt.block_on(C_R.get(&from, &RMockNone {}, &MCMock {}, &MetaMock {}));
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        assert_eq!(result.is_empty(), true);
        // and the repeated call will not call mocker but get from cache
        let result = rt.block_on(C_R.get(&from, &RMockERR {}, &MCMock {}, &MetaMock {}));
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        assert_eq!(result.is_empty(), true);
    }

    struct RMockERR;

    struct RMockERR2;

    struct RMockNone;

    #[async_trait]
    impl RelationDao for RMockERR {
        async fn get_relations<MC, M>(&self, _from: &str, _meta_cache_getter: &MC, _meta_getter: &M) -> Relations where MC: MetaCache, M: MetaDao {
            Err(NatureError::EnvironmentError("can't connect".to_string()))
        }

        async fn insert(&self, _one: RawRelation) -> Result<usize> {
            unimplemented!()
        }

        async fn delete(&self, _one: RawRelation) -> Result<usize> {
            unimplemented!()
        }

        async fn update_flag(&self, _from: &str, _to: &str, _flag_f: i32) -> Result<usize> {
            unimplemented!()
        }

        async fn insert_by_biz(&self, _from: &str, _to: &str, _url: &str, _protocol: &str) -> Result<RawRelation> {
            unimplemented!()
        }

        async fn delete_by_biz(&self, _from: &str, _to: &str) -> Result<usize> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl RelationDao for RMockERR2 {
        async fn get_relations<MC, M>(&self, _from: &str, _meta_cache_getter: &MC, _meta_getter: &M) -> Relations where MC: MetaCache, M: MetaDao {
            Err(NatureError::EnvironmentError("another error".to_string()))
        }

        async fn insert(&self, _one: RawRelation) -> Result<usize> {
            unimplemented!()
        }

        async fn delete(&self, _one: RawRelation) -> Result<usize> {
            unimplemented!()
        }

        async fn update_flag(&self, _from: &str, _to: &str, _flag_f: i32) -> Result<usize> {
            unimplemented!()
        }

        async fn insert_by_biz(&self, _from: &str, _to: &str, _url: &str, _protocol: &str) -> Result<RawRelation> {
            unimplemented!()
        }

        async fn delete_by_biz(&self, _from: &str, _to: &str) -> Result<usize> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl RelationDao for RMockNone {
        async fn get_relations<MC, M>(&self, _from: &str, _meta_cache_getter: &MC, _meta_getter: &M) -> Relations where MC: MetaCache, M: MetaDao {
            Ok(vec![])
        }

        async fn insert(&self, _one: RawRelation) -> Result<usize> {
            unimplemented!()
        }

        async fn delete(&self, _one: RawRelation) -> Result<usize> {
            unimplemented!()
        }

        async fn update_flag(&self, _from: &str, _to: &str, _flag_f: i32) -> Result<usize> {
            unimplemented!()
        }

        async fn insert_by_biz(&self, _from: &str, _to: &str, _url: &str, _protocol: &str) -> Result<RawRelation> {
            unimplemented!()
        }

        async fn delete_by_biz(&self, _from: &str, _to: &str) -> Result<usize> {
            unimplemented!()
        }
    }

    #[derive(Copy, Clone)]
    struct MCMock;

    #[async_trait]
    impl MetaCache for MCMock {
        async fn get<M>(&self, meta_str: &str, _getter: &M) -> Result<Meta> where M: MetaDao {
            Ok(Meta::from_string(meta_str)?)
        }
    }


    #[derive(Copy, Clone)]
    struct MetaMock;

    #[async_trait]
    impl MetaDao for MetaMock {
        async fn get(&self, m: &str) -> Result<Option<RawMeta>> {
            Ok(Some(RawMeta::from(Meta::from_string(m)?)))
        }

        async fn insert(&self, _define: &RawMeta) -> Result<usize> {
            unimplemented!()
        }

        async fn update_flag(&self, _meta_str: &str, _flag_f: i32) -> Result<usize> {
            unimplemented!()
        }

        async fn delete(&self, _m: &Meta) -> Result<usize> {
            unimplemented!()
        }
    }
}
