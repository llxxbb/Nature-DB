use std::ops::Sub;

use chrono::{Local, TimeZone};

use nature_common::{DynamicConverter, Executor, get_para_and_key_from_para, Instance, is_default, Meta, MetaType, Result};

use crate::{MetaCache, MetaDao, Relation};
use crate::flow_tool::ContextChecker;
use crate::flow_tool::StateChecker;
use crate::models::relation_target::RelationTarget;

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Clone, Default)]
pub struct Mission {
    pub to: Meta,
    pub executor: Executor,
    pub filter_before: Vec<Executor>,
    pub filter_after: Vec<Executor>,
    pub target_demand: RelationTarget,
    pub use_upstream_id: bool,
    pub delay: i32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MissionRaw {
    pub to: String,
    pub executor: Executor,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub filter_before: Vec<Executor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub filter_after: Vec<Executor>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub target_demand: RelationTarget,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub use_upstream_id: bool,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub delay: i32,
}

impl From<Mission> for MissionRaw {
    fn from(input: Mission) -> Self {
        MissionRaw {
            to: input.to.meta_string(),
            executor: input.executor,
            filter_before: input.filter_before,
            filter_after: input.filter_after,
            target_demand: input.target_demand,
            use_upstream_id: input.use_upstream_id,
            delay: input.delay,
        }
    }
}

impl MissionRaw {
    pub fn to_json(&self) -> Result<String> {
        let rtn = serde_json::to_string(self)?;
        Ok(rtn)
    }
    pub fn from_json(json: &str) -> Result<Self> {
        let rtn: Self = serde_json::from_str(json)?;
        Ok(rtn)
    }
}

pub type MissionFilter = fn(&Instance, &Vec<Relation>) -> Option<Vec<Mission>>;

impl Mission {
    pub fn for_dynamic(dynamic: Vec<DynamicConverter>) -> Result<Vec<Mission>> {
        debug!("------------------get_dynamic_route------------------------");
        let mut missions: Vec<Mission> = Vec::new();
        for d in dynamic {
            let t = match d.to {
                None => Meta::new("", 1, MetaType::Null)?,
                Some(s) => Meta::new(&s, 1, MetaType::Dynamic)?,
            };
            let mission = Mission {
                to: t,
                executor: d.fun.clone(),
                filter_before: vec![],
                filter_after: vec![],
                target_demand: Default::default(),
                use_upstream_id: d.use_upstream_id,
                delay: d.delay,
            };
            missions.push(mission)
        }
        debug!("missions : {:?}", missions);
        Ok(missions)
    }

    /// Check the instance's context, sys_context and states whether satisfy the Selector request
    pub fn get_by_instance(instance: &Instance, relations: &Vec<Relation>, ctx_chk: ContextChecker, sta_chk: StateChecker) -> Vec<Mission> {
        if relations.is_empty() { return vec![]; }
        let mut rtn: Vec<Mission> = Vec::new();
        for r in relations {
            if r.selector.is_some() {
                let selector = &r.selector.clone().unwrap();
                if !ctx_chk(&instance.data.context, &selector.context_none, &selector.context_all, &selector.context_any) {
                    continue;
                }
                if !ctx_chk(&instance.data.sys_context, &selector.sys_context_none, &selector.sys_context_all, &selector.sys_context_any) {
                    continue;
                }
                // only verify source status, target status will be checked later.
                if !sta_chk(&instance.data.states, &selector.state_none, &selector.state_all, &selector.state_any) {
                    continue;
                }
            }
            let m = Mission {
                to: r.to.clone(),
                executor: r.executor.clone(),
                filter_before: r.filter_before.clone(),
                filter_after: r.filter_after.clone(),
                target_demand: r.target.clone(),
                use_upstream_id: r.use_upstream_id,
                delay: match get_delay(instance, r) {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("relation will be ignored, R: {}, E:{} ", r.relation_string(), e);
                        continue;
                    }
                },
            };
            // debug!("instance meta: {}, selected relation is {}", instance.meta, r.relation_string());
            rtn.push(m);
        }
        rtn
    }

    pub fn from_raw<MC, M>(raw: &MissionRaw, mc_g: &MC, m_g: &M) -> Result<Self>
        where MC: MetaCache, M: MetaDao
    {
        let rtn = Mission {
            to: mc_g.get(&raw.to, m_g)?,
            executor: raw.executor.clone(),
            filter_before: raw.filter_before.clone(),
            filter_after: raw.filter_after.clone(),
            target_demand: raw.target_demand.clone(),
            use_upstream_id: raw.use_upstream_id,
            delay: raw.delay,
        };
        Ok(rtn)
    }
}

fn get_delay(ins: &Instance, rela: &Relation) -> Result<i32> {
    let rtn: i32 = if rela.delay > 0 {
        rela.delay
    } else if rela.delay_on_pare.0 > 0 {
        let rtn = get_para_and_key_from_para(&ins.para, &vec![rela.delay_on_pare.1])?;
        let diff = Local.timestamp_millis(rtn.0.parse::<i64>()?).sub(Local::now()).num_seconds();
        diff as i32 + rela.delay_on_pare.0
    } else {
        0
    };
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use nature_common::TargetState;

    use crate::FlowSelector;
    use crate::models::flow_tool::{context_check, state_check};
    use crate::models::relation_target::RelationTarget;

    use super::*;

    #[test]
    fn get_delay_test() {
        // none delay set
        let mut ins = Instance::default();
        let mut relation = Relation::default();
        let result = get_delay(&ins, &relation).unwrap();
        assert_eq!(result, 0);

        // para delay is set, but para not set
        relation.delay_on_pare = (100, 0);
        let result = get_delay(&ins, &relation);
        assert_eq!(result.is_err(), true);

        // para delay is set
        ins.para = (Local::now().timestamp_millis() + 200000).to_string();
        let result = get_delay(&ins, &relation).unwrap();
        assert_eq!(result >= 299 && result <= 300, true);

        // delay is set, delay is the high priority
        relation.delay = 50;
        ins.para = Local::now().timestamp_millis().to_string();
        let result = get_delay(&ins, &relation).unwrap();
        assert_eq!(result, 50);
    }

    #[test]
    fn state_verify() {
        let mut relation = Relation::default();
        let mut selector = FlowSelector::default();
        selector.state_any.insert("a".to_string());
        relation.selector = Some(selector);
        let relations = vec![relation];
        let mut instance = Instance::default();
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_empty(), true);
        instance.states.insert("a".to_string());
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_empty(), false);
    }

    #[test]
    fn sys_context_verify() {
        let mut relation = Relation::default();
        let mut selector = FlowSelector::default();
        selector.sys_context_any.insert("a".to_string());
        relation.selector = Some(selector);
        let relations = vec![relation];
        let mut instance = Instance::default();
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_empty(), true);
        instance.sys_context.insert("a".to_string(), "x".to_string());
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_empty(), false);
    }

    #[test]
    fn context_verify() {
        let mut relation = Relation::default();
        let mut selector = FlowSelector::default();
        selector.context_any.insert("a".to_string());
        relation.selector = Some(selector);
        let relations = vec![relation];
        let mut instance = Instance::default();
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_empty(), true);
        instance.context.insert("a".to_string(), "x".to_string());
        let rtn = Mission::get_by_instance(&instance, &relations, context_check, state_check);
        assert_eq!(rtn.is_empty(), false);
    }

    #[test]
    fn mission_copy_from_relation() {
        let meta = Meta::from_string("B:hello:1").unwrap();
        let executor = Executor::for_local("abc");
        let mut state = TargetState::default();
        state.add = Some(vec!["a".to_string()]);
        let state = Some(state);
        let target = RelationTarget {
            states: state.clone(),
            upstream_para: vec![],
        };
        let mut relation = Relation::default();
        relation.from = "a".to_string();
        relation.to = meta.clone();
        relation.executor = executor.clone();
        relation.use_upstream_id = true;
        relation.target = target;
        relation.delay = 2;
        let relations = vec![relation];
        let rtn = Mission::get_by_instance(&Instance::default(), &relations, context_check, state_check);
        let rtn = &rtn[0];
        assert_eq!(rtn.delay, 2);
        assert_eq!(rtn.executor, executor);
        assert_eq!(rtn.to, meta);
        assert_eq!(rtn.use_upstream_id, true);
        assert_eq!(rtn.target_demand.states, state);
    }

    #[test]
    fn many_relations() {
        let relations = vec![Relation::default(), Relation::default(), Relation::default()];
        let rtn = Mission::get_by_instance(&Instance::default(), &relations, context_check, state_check);
        assert_eq!(rtn.len(), 3);
    }

    #[test]
    fn one_relation_but_no_selector() {
        let relations = vec![Relation::default()];
        let rtn = Mission::get_by_instance(&Instance::default(), &relations, context_check, state_check);
        assert_eq!(rtn.len(), 1);
    }

    #[test]
    fn no_relation() {
        let rtn = Mission::get_by_instance(&Instance::default(), &vec![], context_check, state_check);
        assert_eq!(rtn.is_empty(), true);
    }
}
