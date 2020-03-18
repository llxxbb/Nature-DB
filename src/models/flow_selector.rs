use std::collections::{HashMap, HashSet};

/// none: the highest priority to process, means can't include any one
/// all : the middle priority to process, means must include all
/// any: the lowest priority to process, means must include one
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FlowSelector {
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_state_all: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_state_any: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_state_none: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_state_all: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_state_any: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_state_none: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_all: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_any: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_none: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub sys_context_all: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub sys_context_any: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub sys_context_none: HashSet<String>,
}

impl FlowSelector {
    pub fn context_check(&self, contexts: &HashMap<String, String>) -> bool {
        for exclude in &self.context_none {
            if contexts.contains_key(exclude) {
                return false;
            }
        }
        for include in &self.context_all {
            if !contexts.contains_key(include) {
                return false;
            }
        }
        if self.context_any.is_empty() {
            return true;
        }
        for any in &self.context_any {
            if contexts.contains_key(any) {
                return true;
            }
        }
        false
    }

    pub fn sys_context_check(&self, contexts: &HashMap<String, String>) -> bool {
        for exclude in &self.sys_context_none {
            if contexts.contains_key(exclude) {
                return false;
            }
        }
        for include in &self.sys_context_all {
            if !contexts.contains_key(include) {
                return false;
            }
        }
        if self.sys_context_any.is_empty() {
            return true;
        }
        for any in &self.sys_context_any {
            if contexts.contains_key(any) {
                return true;
            }
        }
        false
    }

    pub fn source_status_check(&self, status: &HashSet<String>) -> bool {
        for exclude in &self.source_state_none {
            if status.contains(exclude) {
                return false;
            }
        }
        for include in &self.source_state_all {
            if !status.contains(include) {
                return false;
            }
        }
        if self.source_state_any.is_empty() {
            return true;
        }
        for any in &self.source_state_any {
            if status.contains(any) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn selector_serder_test() {
        let mut se = FlowSelector {
            source_state_all: HashSet::new(),
            source_state_any: Default::default(),
            source_state_none: HashSet::new(),
            target_state_all: HashSet::new(),
            target_state_any: Default::default(),
            target_state_none: HashSet::new(),
            context_all: HashSet::new(),
            context_any: Default::default(),
            context_none: HashSet::new(),
            sys_context_all: Default::default(),
            sys_context_any: Default::default(),
            sys_context_none: Default::default(),
        };
        // test for null
        let rtn = serde_json::to_string(&se);
        assert_eq!(rtn.unwrap(), "{}");

        // test for some value
        se.source_state_all.insert("aaa".to_string());
        let rtn = serde_json::to_string(&se).unwrap();
        assert_eq!(rtn, "{\"source_status_include\":[\"aaa\"]}");

        // deserialize

        let de: FlowSelector = serde_json::from_str(&rtn).unwrap();
        assert_eq!(de.context_none.is_empty(), true);
        assert_eq!(de.source_state_all.len(), 1);
    }
}
