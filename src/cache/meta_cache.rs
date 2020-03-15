extern crate r2d2;

use std::convert::TryInto;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::{Meta, MetaType, NatureError, Result};

use crate::MetaGetter;

lazy_static! {
    static ref CACHE: Mutex<LruCache<String, Meta>> = Mutex::new(LruCache::<String, Meta>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type MetaCacheGetter = fn(&str, MetaGetter) -> Result<Meta>;

pub struct MetaCacheImpl;

impl MetaCacheImpl {
    pub fn get(meta_str: &str, getter: MetaGetter) -> Result<Meta> {
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
                    MetaType::Multi => cache_sub_metas(meta_str, m),
                    _ => {
                        let error = NatureError::VerifyError(format!("{} not defined", meta_str));
                        warn!("{}", error);
                        Err(error)
                    }
                }
            }
            Some(def) => {
                let meta: Meta = def.try_into()?;
                let _ = check_master(&meta, getter)?;
                let mut cache = CACHE.lock().unwrap();
                cache.insert(meta_str.to_string(), meta.clone());
                Ok(meta)
            }
        }
    }
}

fn cache_sub_metas(meta_str: &str, m: Meta) -> Result<Meta> {
    let mut cache = CACHE.lock().unwrap();
    cache.insert(meta_str.to_string(), m.clone());
    match m.get_setting() {
        None => Ok(m),
        Some(setting) => {
            match setting.multi_meta {
                None => Ok(m),
                Some(multi) => {
                    let s_meta = multi.get_metas();
                    s_meta.into_iter().for_each(|one| {
                        let key = one.meta_string();
                        cache.insert(key, one);
                    });
                    Ok(m)
                }
            }
        }
    }
}

fn check_master(meta: &Meta, getter: MetaGetter) -> Result<()> {
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
    use nature_common::{MetaSetting, MultiMetaSetting};

    use super::*;

    #[test]
    fn cache_sub_meta_test() {
        let multi_meta = MultiMetaSetting::new("M:parent", "p", 1, vec!["a".to_string(), "b".to_string()], Default::default());
        let setting = MetaSetting {
            is_state: false,
            master: None,
            multi_meta: Some(multi_meta.unwrap()),
            conflict_avoid: false,
        };
        let mut m = Meta::from_string("B:test:3").unwrap();
        let _ = m.set_setting(&serde_json::to_string(&setting).unwrap());
        {
            let mut c = CACHE.lock().unwrap();
            c.clear();
        }
        let rtn = cache_sub_metas("test", m).unwrap();
        assert_eq!(rtn.meta_string(), "B:test:3");
        let mut c = CACHE.lock().unwrap();
        let x = c.get("test");
        assert_eq!(x.is_some(), true);
        let x = c.get("B:p/a:1");
        assert_eq!(x.is_some(), true);
        let x = c.get("B:p/b:1");
        assert_eq!(x.is_some(), true);
    }
}