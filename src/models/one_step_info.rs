use std::clone::Clone;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::ops::Range;
use std::ptr;
use std::string::ToString;

use rand::{Rng, thread_rng};

use nature_common::{Executor, Meta, NatureError, Protocol, Result, TargetState};

use crate::{FlowSelector, MetaCacheGetter, MetaGetter, OneStepFlowSettings, RawOneStepFlow};

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OneStepFlow {
    pub from: Meta,
    pub to: Meta,
    pub selector: Option<FlowSelector>,
    pub executor: Executor,
    pub use_upstream_id: bool,
    pub target_states: Option<TargetState>,
}

impl Iterator for OneStepFlow {
    type Item = OneStepFlow;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.clone())
    }
}

impl OneStepFlow {
    pub fn from_raw(val: RawOneStepFlow, meta_cache_getter: MetaCacheGetter, meta_getter: MetaGetter) -> Result<Vec<OneStepFlow>> {
        let settings = serde_json::from_str::<OneStepFlowSettings>(&val.settings)?;
        let selector = &settings.selector;
        let mut group = String::new();
        let id = uuid::Uuid::new_v4().to_string();
        // check group: only one group is allowed.
        let mut group_check = true;
        settings.executor.iter().for_each(|e| {
            if group.is_empty() {
                group = e.group.clone();
                if group.is_empty() {
                    group = id.clone()
                }
            } else if group != e.group {
                group_check = false;
            }
        });
        if !group_check {
            return Err(NatureError::VerifyError("in one setting all executor's group must be same.".to_string()));
        }
        let use_upstream_id = settings.use_upstream_id;
        let m_from = Meta::from_string(&val.from_meta)?;
        let m_to = OneStepFlow::check_converter(&val.to_meta, meta_cache_getter, meta_getter, &settings)?;
        let rtn = settings.executor.iter().map(|e| {
            let mut e2 = e.clone();
            e2.group = group.clone();
            OneStepFlow {
                from: m_from.clone(),
                to: m_to.clone(),
                selector: selector.clone(),
                executor: e2,
                use_upstream_id,
                target_states: settings.target_states.clone(),
            }
        }).collect();
        Ok(rtn)
    }

    fn check_converter(meta_to: &str, meta_cache_getter: MetaCacheGetter, meta_getter: MetaGetter, settings: &OneStepFlowSettings) -> Result<Meta> {
        let m_to = Meta::from_string(meta_to)?;
        let m_to = meta_cache_getter(&m_to, meta_getter)?;
        let m_to = m_to.try_into()?;
        if let Some(ts) = &settings.target_states {
            if let Some(x) = &ts.add {
                OneStepFlow::check_state(&m_to, x)?
            };
            if let Some(x) = &ts.remove {
                OneStepFlow::check_state(&m_to, x)?
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

    pub fn weight_filter(relations: &[OneStepFlow], balances: &HashMap<Executor, Range<f32>>) -> Vec<OneStepFlow> {
        let mut rtn: Vec<OneStepFlow> = Vec::new();
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
    pub fn weight_calculate(groups: &HashMap<String, Vec<OneStepFlow>>) -> HashMap<Executor, Range<f32>> {
        let mut rtn: HashMap<Executor, Range<f32>> = HashMap::new();
        for group in groups.values() {
            // summarize the weight for one group
            let sum = group.iter().fold(0u32, |sum, mapping| {
                sum + mapping.executor.proportion
            });
            if sum == 0 {
                continue;
            }
            // give a certain range for every participants
            let mut begin = 0.0;
            let last = group.last().unwrap();
            for m in group {
                let w = m.executor.proportion as f32 / sum as f32;
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
    pub fn get_label_groups(maps: &[OneStepFlow]) -> HashMap<String, Vec<OneStepFlow>> {
        let mut labels: HashMap<String, Vec<OneStepFlow>> = HashMap::new();
        for mapping in maps {
            let mappings = labels.entry(mapping.executor.group.clone()).or_insert_with(Vec::new);
            mappings.push(mapping.clone());
        }
        labels
    }

    pub fn new_for_local_executor(from: &str, to: &str, local_executor: &str) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: "".to_string(),
                proportion: 1,
            },
            use_upstream_id: false,
            target_states: None,
        })
    }
    pub fn new_for_local_executor_with_group_and_proportion(from: &str, to: &str, local_executor: &str, group: &str, proportion: u32) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: group.to_string(),
                proportion,
            },
            use_upstream_id: false,
            target_states: None,
        })
    }
    pub fn new_for_source_status_needed(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(FlowSelector {
                source_status_include: set.clone(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }
    pub fn new_for_source_status_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(FlowSelector {
                source_status_include: HashSet::new(),
                source_status_exclude: set.clone(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }
    pub fn new_for_context_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(FlowSelector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: set.clone(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }
    pub fn new_for_context_include(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Meta::new(from)?,
            to: Meta::new(to)?,
            selector: Some(FlowSelector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: set.clone(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
            use_upstream_id: false,
            target_states: None,
        })
    }
}

#[cfg(test)]
mod test_from_raw {
    use crate::RawMeta;

    use super::*;

    #[test]
    fn one_group_is_ok() {
        let settings = OneStepFlowSettings {
            selector: None,
            executor: vec![
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_one".to_string(),
                    group: "grp_one".to_string(),
                    proportion: 100,
                },
            ],
            use_upstream_id: false,
            target_states: None,
        };
        let raw = RawOneStepFlow {
            from_meta: "from".to_string(),
            to_meta: "to".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
        };
        let rtn = OneStepFlow::from_raw(raw, meta_cache, meta);
        assert_eq!(rtn.is_ok(), true);
    }

    #[test]
    fn multiple_group_is_illegal() {
        let settings = OneStepFlowSettings {
            selector: None,
            executor: vec![
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_one".to_string(),
                    group: "grp_one".to_string(),
                    proportion: 100,
                },
                Executor {
                    protocol: Protocol::LocalRust,
                    url: "url_two".to_string(),
                    group: "url_two".to_string(),
                    proportion: 200,
                },
            ],
            use_upstream_id: false,
            target_states: None,
        };
        let raw = RawOneStepFlow {
            from_meta: "from".to_string(),
            to_meta: "to".to_string(),
            settings: serde_json::to_string(&settings).unwrap(),
            flag: 1,
        };
        let rtn = OneStepFlow::from_raw(raw, meta_cache, meta);
        assert_eq!(rtn, Err(NatureError::VerifyError("in one setting all executor's grpup must be same.".to_string())));
    }

    fn meta_cache(m: &Meta, _: MetaGetter) -> Result<RawMeta> {
        Ok(RawMeta::from(m.clone()))
    }

    fn meta(m: &Meta) -> Result<Option<RawMeta>> {
        Ok(Some(RawMeta::from(m.clone())))
    }
}