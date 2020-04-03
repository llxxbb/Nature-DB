use std::clone::Clone;
use std::collections::HashMap;
use std::ops::Range;
use std::ptr;
use std::string::ToString;

use rand::{Rng, thread_rng};

use nature_common::{Executor, Meta, NatureError, Protocol, Result, TargetState};

use crate::{FlowSelector, MetaCacheGetter, MetaGetter, RawRelation, RelationSettings};

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Relation {
    pub from: String,
    pub to: Meta,
    pub selector: Option<FlowSelector>,
    pub executor: Executor,
    pub use_upstream_id: bool,
    pub target_states: Option<TargetState>,
    pub delay: i32,
}

impl Iterator for Relation {
    type Item = Relation;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.clone())
    }
}

impl Relation {
    pub fn from_raw(val: RawRelation, meta_cache_getter: MetaCacheGetter, meta_getter: &MetaGetter) -> Result<Vec<Relation>> {
        let settings = match serde_json::from_str::<RelationSettings>(&val.settings) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("{}'s setting format error: {:?}", val.get_string(), e);
                warn!("{}", &msg);
                return Err(NatureError::VerifyError(msg));
            }
        };
        let selector = &settings.selector;
        let m_to = Relation::check_converter(&val.to_meta, meta_cache_getter, meta_getter, &settings)?;
        let mut group = String::new();
        let mut rtn: Vec<Relation> = vec![];
        let mut err: String = String::new();
        if let Some(e) = &settings.executor {
            let find = e.iter().find(|e| {
                // check Protocol type
                if e.protocol == Protocol::Auto {
                    err = format!("{} Protocol::Auto can not be used by user. ", val.get_string());
                    return true;
                }
                // check group: only one group is allowed.
                if group.is_empty() {
                    group = e.group.clone();
                    if group.is_empty() {
                        group = uuid::Uuid::new_v4().to_string();
                    }
                } else if group != e.group {
                    err = "in one setting all executor's group must be same.".to_string();
                    return true;
                }
                // generate relation
                let mut e2 = (*e).clone();
                e2.group = group.clone();
                let r = Relation {
                    from: val.from_meta.to_string(),
                    to: m_to.clone(),
                    selector: selector.clone(),
                    executor: e2,
                    use_upstream_id: settings.use_upstream_id,
                    target_states: settings.target_states.clone(),
                    delay: settings.delay,
                };
                rtn.push(r);
                // return find result
                false
            });
            if find.is_some() {
                warn!("{}", &err);
                return Err(NatureError::VerifyError(err));
            }
        } else {
            if let Some(s) = &m_to.get_setting() {
                if s.master.is_some() {
                    let r = Relation {
                        from: val.from_meta.to_string(),
                        to: m_to.clone(),
                        selector: selector.clone(),
                        executor: Executor::new_auto(),
                        use_upstream_id: settings.use_upstream_id,
                        target_states: settings.target_states.clone(),
                        delay: settings.delay,
                    };
                    rtn.push(r);
                }
            }
        }
        debug!("load {}", val.get_string());
        Ok(rtn)
    }

    fn check_converter(meta_to: &str, meta_cache_getter: MetaCacheGetter, meta_getter: &MetaGetter, settings: &RelationSettings) -> Result<Meta> {
        let m_to = meta_cache_getter(meta_to, &meta_getter)?;
        if let Some(ts) = &settings.target_states {
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

    pub fn weight_filter(relations: &[Relation], balances: &HashMap<Executor, Range<f32>>) -> Vec<Relation> {
        let mut rtn: Vec<Relation> = Vec::new();
        let rnd = thread_rng().gen::<f32>();
        for m in relations {
            match balances.get(&m.executor) {
                Some(rng) => if rng.contains(&rnd) {
                    rtn.push(m.clone());
                },
                None => rtn.push(m.clone())
            };
        }
        rtn
    }

    /// weight group will be cached
    pub fn weight_calculate(groups: &HashMap<String, Vec<Relation>>) -> HashMap<Executor, Range<f32>> {
        let mut rtn: HashMap<Executor, Range<f32>> = HashMap::new();
        for group in groups.values() {
            // summarize the weight for one group
            let sum = group.iter().fold(0u32, |sum, mapping| {
                sum + mapping.executor.weight
            });
            if sum == 0 {
                continue;
            }
            // give a certain range for every participants
            let mut begin = 0.0;
            let last = group.last().unwrap();
            for m in group {
                let w = m.executor.weight as f32 / sum as f32;
                let end = begin + w;
                if ptr::eq(m, last) {
                    // last must great 1
                    rtn.insert(m.executor.clone(), begin..1.1);
                } else {
                    rtn.insert(m.executor.clone(), begin..end);
                }
                begin = end;
            }
        }
        rtn
    }

    /// group by labels. Only one flow will be used when there are same label. This can be used to switch two different flows smoothly.
    pub fn get_label_groups(maps: &[Relation]) -> HashMap<String, Vec<Relation>> {
        let mut labels: HashMap<String, Vec<Relation>> = HashMap::new();
        for mapping in maps {
            let mappings = labels.entry(mapping.executor.group.clone()).or_insert_with(Vec::new);
            mappings.push(mapping.clone());
        }
        labels
    }

    pub fn relation_string(&self) -> String {
        format!("{}->{}", self.from, self.to.meta_string()).to_owned()
    }
}

#[cfg(test)]
mod test_from_raw {
    use nature_common::Protocol;

    use crate::RawMeta;

    use super::*;

    #[test]
    fn setting_error_test() {
        let raw = RawRelation {
            from_meta: "B:from:1".to_string(),
            to_meta: "B:to:1".to_string(),
            settings: "dd".to_string(),
            flag: 1,
        };
        let mg: MetaGetter = meta;
        let rtn = Relation::from_raw(raw, meta_cache, &mg);
        assert_eq!(rtn.err().unwrap().to_string().contains("relation[B:from:1->B:to:1]"), true);
    }

    #[test]
    fn one_group_is_ok() {
        let settings = RelationSettings {
            selector: None,
            executor: Some(vec![
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_one".to_string(),
                    group: "grp_one".to_string(),
                    weight: 100,
                },
            ]),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        let raw = RawRelation {
            from_meta: "B:from:1".to_string(),
            to_meta: "B:to:1".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
        };
        let mg: MetaGetter = meta;
        let rtn = Relation::from_raw(raw, meta_cache, &mg);
        assert_eq!(rtn.is_ok(), true);
    }

    #[test]
    fn multiple_group_is_illegal() {
        let settings = RelationSettings {
            selector: None,
            executor: Some(vec![
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_one".to_string(),
                    group: "".to_string(),
                    weight: 100,
                },
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_two".to_string(),
                    group: "url_two".to_string(),
                    weight: 200,
                },
            ]),
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        };
        let raw = RawRelation {
            from_meta: "B:from:1".to_string(),
            to_meta: "B:to:1".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
        };
        let mg: MetaGetter = meta;
        let rtn = Relation::from_raw(raw, meta_cache, &mg);
        assert_eq!(rtn, Err(NatureError::VerifyError("in one setting all executor's group must be same.".to_string())));
    }

    fn meta_cache(m: &str, _: &MetaGetter) -> Result<Meta> {
        Meta::from_string(m)
    }

    fn meta(m: &str) -> Result<Option<RawMeta>> {
        Ok(Some(RawMeta::from(Meta::from_string(m)?)))
    }
}