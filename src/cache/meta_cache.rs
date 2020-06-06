use std::convert::TryInto;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;
use tokio::runtime::Runtime;

use nature_common::{Meta, MetaType, NatureError, Result};

use crate::MetaDao;

lazy_static! {
    pub static ref C_M: MetaCacheImpl = MetaCacheImpl {};
    static ref CACHE: Mutex<LruCache<String, Meta>> = Mutex::new(LruCache::<String, Meta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub trait MetaCache: Copy + Sync + Send {
    fn get<M>(&self, meta_str: &str, getter: &M) -> Result<Meta> where M: MetaDao;
}

#[derive(Copy, Clone)]
pub struct MetaCacheImpl;

impl MetaCache for MetaCacheImpl {
    fn get<M>(&self, meta_str: &str, getter: &M) -> Result<Meta> where M: MetaDao {
        let mut runtime = match Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                let msg = format!("get tokio runtime error : {}", e.to_string());
                warn!("{}", msg);
                return Err(NatureError::LogicalError(msg));
            }
        };
        runtime.block_on(inner_get(meta_str, getter))
    }
}

async fn inner_get<M>(meta_str: &str, getter: &M) -> Result<Meta>
    where M: MetaDao
{
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
    match getter.get(meta_str).await? {
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
            match meta.get_meta_type() {
                MetaType::Multi => {
                    let _ = cache_sub_metas(meta_str, &meta, getter);
                    Ok(meta)
                }
                _ => {
                    let _ = verify_and_load_master(&meta, getter).await?;
                    let mut cache = CACHE.lock().unwrap();
                    cache.insert(meta_str.to_string(), meta.clone());
                    Ok(meta)
                }
            }
        }
    }
}

fn cache_sub_metas<M>(meta_str: &str, m: &Meta, getter: &M) -> Result<()>
    where M: MetaDao
{
    {
        // unlock the cache
        let mut cache = CACHE.lock().unwrap();
        cache.insert(meta_str.to_string(), m.clone());
    }
    match m.get_setting() {
        None => Err(NatureError::VerifyError("Multi-Meta must define sub-metas".to_string())),
        Some(setting) => {
            match setting.multi_meta.len() {
                0 => Err(NatureError::VerifyError("sub-meta number should great than 0".to_string())),
                _n => {
                    for one in setting.multi_meta {
                        let _rtn = C_M.get(&one, getter);
                    }
                    Ok(())
                }
            }
        }
    }
}

async fn verify_and_load_master<M>(meta: &Meta, getter: &M) -> Result<()>
    where M: MetaDao
{
    match meta.get_setting() {
        None => Ok(()),
        Some(setting) => match setting.master {
            None => Ok(()),
            Some(master) => {
                let _ = C_M.get(&master, getter)?;
                Ok(())
            }
        },
    }
}

#[cfg(test)]
mod test {
    use std::collections::btree_set::BTreeSet;

    use nature_common::MetaSetting;

    use crate::RawMeta;

    use super::*;

    #[test]
    fn cache_sub_meta_test() {
        let mut set: BTreeSet<String> = BTreeSet::new();
        set.insert("B:p/a:1".to_owned());
        set.insert("B:p/b:1".to_owned());
        let setting = MetaSetting {
            is_state: false,
            master: None,
            multi_meta: set,
            cache_saved: false,
        };
        let mut m = Meta::from_string("B:test:3").unwrap();
        let _ = m.set_setting(&setting.to_json().unwrap());
        {
            let mut c = CACHE.lock().unwrap();
            c.clear();
        }
        {
            let mut c = CACHE.lock().unwrap();
            let x = c.get("test");
            assert_eq!(x.is_some(), false);
            let x = c.get("B:p/a:1");
            assert_eq!(x.is_some(), false);
            let x = c.get("B:p/b:1");
            assert_eq!(x.is_some(), false);
        }
        let _rtn = cache_sub_metas("test", &m, &MetaMock {}).unwrap();
        {
            let mut c = CACHE.lock().unwrap();
            let x = c.get("test");
            assert_eq!(x.is_some(), true);
            let x = c.get("B:p/a:1");
            assert_eq!(x.is_some(), true);
            let x = c.get("B:p/b:1");
            assert_eq!(x.is_some(), true);
        }
    }

    #[derive(Copy, Clone)]
    struct MetaMock;

    #[async_trait]
    impl MetaDao for MetaMock {
        async fn get(&self, _m: &str) -> Result<Option<RawMeta>> {
            Ok(Some({
                let mut rtn = RawMeta::default();
                rtn.meta_key = "lxb".to_string();
                rtn
            }))
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