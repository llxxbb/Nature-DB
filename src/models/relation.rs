use std::clone::Clone;
use std::string::ToString;

use nature_common::{Executor, Meta, NatureError, Protocol, Result};

use crate::{FlowSelector, MetaCache, MetaDao, RawRelation, RelationSettings};
use crate::models::relation_target::RelationTarget;

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Relation {
    pub from: String,
    pub to: Meta,
    pub selector: Option<FlowSelector>,
    pub executor: Executor,
    pub filter_before: Vec<Executor>,
    pub filter_after: Vec<Executor>,
    pub use_upstream_id: bool,
    pub target: RelationTarget,
    pub delay: i32,
    pub delay_on_pare: (i32, u8),
    pub id_bridge: bool,
}

impl Iterator for Relation {
    type Item = Relation;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.clone())
    }
}

impl Relation {
    pub async fn from_raw<MC, M>(val: RawRelation, meta_cache_getter: &MC, meta_getter: &M) -> Result<Relation>
        where MC: MetaCache, M: MetaDao
    {
        let settings = match serde_json::from_str::<RelationSettings>(&val.settings) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("{}'s setting format error: {:?}", val.get_string(), e);
                warn!("{}", &msg);
                return Err(NatureError::VerifyError(msg));
            }
        };
        let selector = &settings.selector;
        let m_to = Relation::check_converter(&val.to_meta, meta_cache_getter, meta_getter, &settings).await?;
        let rtn = match settings.executor {
            Some(e) => {
                // check Protocol type
                if e.protocol == Protocol::Auto {
                    let err = format!("{} Protocol::Auto can not be used by user. ", val.get_string());
                    return Err(NatureError::VerifyError(err));
                }
                Relation {
                    from: val.from_meta.to_string(),
                    to: m_to,
                    selector: selector.clone(),
                    executor: e,
                    filter_before: settings.filter_before,
                    filter_after: settings.filter_after,
                    use_upstream_id: settings.use_upstream_id,
                    target: settings.target.clone(),
                    delay: settings.delay,
                    delay_on_pare: settings.delay_on_para,
                    id_bridge: settings.id_bridge,
                }
            }
            None => {
                match &m_to.get_setting() {
                    Some(s) => {
                        if s.master.is_some() {
                            Relation {
                                from: val.from_meta.to_string(),
                                to: m_to.clone(),
                                selector: selector.clone(),
                                executor: Executor::new_auto(),
                                filter_before: settings.filter_before,
                                filter_after: settings.filter_after,
                                use_upstream_id: settings.use_upstream_id,
                                target: settings.target.clone(),
                                delay: settings.delay,
                                delay_on_pare: settings.delay_on_para,
                                id_bridge: settings.id_bridge,
                            }
                        } else {
                            return Err(NatureError::VerifyError("master or executor should be defined".to_string()));
                        }
                    }
                    None => return Err(NatureError::VerifyError("master or executor should be defined".to_string()))
                }
            }
        };
        debug!("load {}", val.get_string());
        Ok(rtn)
    }

    async fn check_converter<MC, M>(meta_to: &str, meta_cache_getter: &MC, meta_getter: &M, settings: &RelationSettings) -> Result<Meta>
        where MC: MetaCache, M: MetaDao
    {
        let m_to = meta_cache_getter.get(meta_to, meta_getter).await?;
        if let Some(ts) = &settings.target.states {
            if let Some(x) = &ts.add {
                Relation::check_state(&m_to, x)?
            };
            if let Some(x) = &ts.remove {
                Relation::check_state(&m_to, x)?
            };
        }
        Ok(m_to)
    }

    fn check_state(m_to: &Meta, x: &Vec<String>) -> Result<()> {
        let b = x.iter().filter(|one| { !m_to.has_state_name(one) }).collect::<Vec<&String>>();
        if b.len() > 0 {
            return Err(NatureError::VerifyError(format!("[to meta] did not defined state : {:?} ", b)));
        }
        Ok(())
    }

    pub fn relation_string(&self) -> String {
        format!("{}->{}", self.from, self.to.meta_string()).to_owned()
    }
}

#[cfg(test)]
mod test_from_raw {
    use tokio::runtime::Runtime;

    use nature_common::Protocol;

    use crate::RawMeta;

    use super::*;

    #[test]
    fn master_should_have_relation() {
        let raw = RawRelation {
            from_meta: "B:from:1".to_string(),
            to_meta: "B:to:1".to_string(),
            settings: "{}".to_string(),
            flag: 1,
        };
        let mg = MetaMock {};
        let mut rt = Runtime::new().unwrap();
        let rtn = rt.block_on(Relation::from_raw(raw, &MetaCacheMasterMock {}, &mg)).unwrap();
        assert_eq!(rtn.executor.protocol, Protocol::Auto);
    }

    #[test]
    fn setting_error_test() {
        let raw = RawRelation {
            from_meta: "B:from:1".to_string(),
            to_meta: "B:to:1".to_string(),
            settings: "dd".to_string(),
            flag: 1,
        };
        let mg = MetaMock {};
        let mut rt = Runtime::new().unwrap();
        let rtn = rt.block_on(Relation::from_raw(raw, &MetaCacheMock {}, &mg));
        assert_eq!(rtn.err().unwrap().to_string().contains("relation[B:from:1  --->  B:to:1]"), true);
    }

    #[test]
    fn one_group_is_ok() {
        let settings = RelationSettings {
            selector: None,
            executor: Some(Executor {
                protocol: Protocol::LocalRust,
                url: "url_one".to_string(),
                settings: "".to_string(),
            }),
            filter_before: vec![],
            filter_after: vec![],
            use_upstream_id: false,
            target: Default::default(),
            delay: 0,
            delay_on_para: (0, 0),
            id_bridge: false
        };
        let raw = RawRelation {
            from_meta: "B:from:1".to_string(),
            to_meta: "B:to:1".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
        };
        let mg = MetaMock {};
        let mut rt = Runtime::new().unwrap();
        let rtn = rt.block_on(Relation::from_raw(raw, &MetaCacheMock {}, &mg));
        assert_eq!(rtn.is_ok(), true);
    }

    #[derive(Copy, Clone)]
    struct MetaCacheMasterMock;

    #[async_trait]
    impl MetaCache for MetaCacheMasterMock {
        async fn get<M>(&self, m: &str, _getter: &M) -> Result<Meta> where M: MetaDao {
            if m.eq("B:to:1") {
                let mut rtn = Meta::from_string(m).unwrap();
                let _ = rtn.set_setting(r#"{"master":"B:from:1"}"#);
                Ok(rtn)
            } else {
                Meta::from_string(m)
            }
        }
    }

    #[derive(Copy, Clone)]
    struct MetaCacheMock;

    #[async_trait]
    impl MetaCache for MetaCacheMock {
        async fn get<M>(&self, meta_str: &str, _getter: &M) -> Result<Meta> where M: MetaDao {
            Meta::from_string(meta_str)
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