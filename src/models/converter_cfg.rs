use std::collections::HashSet;
use std::iter::Iterator;

use nature_common::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub instance: Instance,
    pub maps: Vec<OneStepFlow>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LastStatusDemand {
    pub target_status_include: HashSet<String>,
    pub target_status_exclude: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConverterInfo {
    pub from: Instance,
    pub target: Mission,
    pub last_status: Option<Instance>,
}

impl Default for ConverterInfo {
    fn default() -> Self {
        ConverterInfo {
            from: Instance::default(),
            target: Mission::default(),
            last_status: None,
        }
    }
}

/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Mission {
    pub to: Thing,
    pub executor: Executor,
    pub last_status_demand: Option<LastStatusDemand>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Selector {
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_status_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_status_exclude: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_status_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_status_exclude: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_exclude: HashSet<String>,
}


/// the compose of `Mapping::from`, `Mapping::to` and `Weight::label` must be unique
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OneStepFlow {
    pub from: Thing,
    pub to: Thing,
    pub selector: Option<Selector>,
    pub executor: Executor,
}

impl Iterator for OneStepFlow {
    type Item = OneStepFlow;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.clone())
    }
}

impl OneStepFlow {
    pub fn new_for_local_executor(from: &str, to: &str, local_executor: &str) -> Result<Self> {
        Ok(OneStepFlow {
            from: Thing::new(from)?,
            to: Thing::new(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: "".to_string(),
                proportion: 1,
            },
        })
    }
    pub fn new_for_local_executor_with_group_and_proportion(from: &str, to: &str, local_executor: &str, group: &str, proportion: u32) -> Result<Self> {
        Ok(OneStepFlow {
            from: Thing::new(from)?,
            to: Thing::new(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: group.to_string(),
                proportion,
            },
        })
    }
    pub fn new_for_source_status_needed(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Thing::new(from)?,
            to: Thing::new(to)?,
            selector: Some(Selector {
                source_status_include: set.clone(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
        })
    }
    pub fn new_for_source_status_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Thing::new(from)?,
            to: Thing::new(to)?,
            selector: Some(Selector {
                source_status_include: HashSet::new(),
                source_status_exclude: set.clone(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
        })
    }
    pub fn new_for_context_excluded(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Thing::new(from)?,
            to: Thing::new(to)?,
            selector: Some(Selector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: HashSet::new(),
                context_exclude: set.clone(),
            }),
            executor: Executor::default(),
        })
    }
    pub fn new_for_context_include(from: &str, to: &str, set: &HashSet<String>) -> Result<Self> {
        Ok(OneStepFlow {
            from: Thing::new(from)?,
            to: Thing::new(to)?,
            selector: Some(Selector {
                source_status_include: HashSet::new(),
                source_status_exclude: HashSet::new(),
                target_status_include: HashSet::new(),
                target_status_exclude: HashSet::new(),
                context_include: set.clone(),
                context_exclude: HashSet::new(),
            }),
            executor: Executor::default(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OneStepFlowSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub selector: Option<Selector>,
    pub executor: Vec<Executor>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn selector_serder_test() {
        let mut se = Selector {
            source_status_include: HashSet::new(),
            source_status_exclude: HashSet::new(),
            target_status_include: HashSet::new(),
            target_status_exclude: HashSet::new(),
            context_include: HashSet::new(),
            context_exclude: HashSet::new(),
        };
        // test for null
        let rtn = serde_json::to_string(&se);
        assert_eq!(rtn.unwrap(), "{}");

        // test for some value
        se.source_status_include.insert("aaa".to_string());
        let rtn = serde_json::to_string(&se).unwrap();
        assert_eq!(rtn, "{\"source_status_include\":[\"aaa\"]}");

        // deserialize

        let de: Selector = serde_json::from_str(&rtn).unwrap();
        assert_eq!(de.context_exclude.is_empty(), true);
        assert_eq!(de.source_status_include.len(), 1);
    }

    #[test]
    fn none_for_selector_one_step_flow_settings() {
        let setting = OneStepFlowSettings {
            selector: None,
            executor: vec![],
        };
        let result = serde_json::to_string(&setting).unwrap();
        let res_str = r#"{"executor":[]}"#;
        assert_eq!(result, res_str);
        let res_obj: OneStepFlowSettings = serde_json::from_str(res_str).unwrap();
        assert_eq!(res_obj, setting);
    }
}