use std::convert::TryInto;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::{Meta, MetaType, NatureError, Result};

use crate::MetaGetter;

lazy_static! {
    static ref CACHE: Mutex<LruCache<String, Meta>> = Mutex::new(LruCache::<String, Meta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type MetaCacheGetter = fn(&str, &MetaGetter) -> Result<Meta>;

pub static MCG : MetaCacheGetter = MetaCacheImpl::get;

pub struct MetaCacheImpl;

impl MetaCacheImpl {
    pub fn get(meta_str: &str, getter: &MetaGetter) -> Result<Meta> {
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
        match getter(meta_str)? {
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
                        let _ = verify_and_load_master(&meta, getter)?;
                        let mut cache = CACHE.lock().unwrap();
                        cache.insert(meta_str.to_string(), meta.clone());
                        Ok(meta)
                    }
                }
            }
        }
    }
}

fn cache_sub_metas(meta_str: &str, m: &Meta, getter: &MetaGetter) -> Result<()> {
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
                    setting.multi_meta.into_iter().for_each(|one| {
                        let _rtn = MetaCacheImpl::get(&one, getter);
                    });
                    Ok(())
                }
            }
        }
    }
}

fn verify_and_load_master(meta: &Meta, getter: &MetaGetter) -> Result<()> {
    match meta.get_setting() {
        None => Ok(()),
        Some(setting) => match setting.master {
            None => Ok(()),
            Some(master) => {
                let _ = MetaCacheImpl::get(&master, getter)?;
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
            conflict_avoid: false,
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
        let _rtn = cache_sub_metas("test", &m, &(get as MetaGetter)).unwrap();
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

    fn get(_meta_str: &str) -> Result<Option<RawMeta>> {
        Ok(Some({
            let mut rtn = RawMeta::default();
            rtn.meta_key = "lxb".to_string();
            rtn
        }))
    }
}