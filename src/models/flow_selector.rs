use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FlowSelector {
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_state_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub source_state_exclude: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_state_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub target_state_exclude: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_include: HashSet<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    pub context_exclude: HashSet<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn selector_serder_test() {
        let mut se = FlowSelector {
            source_state_include: HashSet::new(),
            source_state_exclude: HashSet::new(),
            target_state_include: HashSet::new(),
            target_state_exclude: HashSet::new(),
            context_include: HashSet::new(),
            context_exclude: HashSet::new(),
        };
        // test for null
        let rtn = serde_json::to_string(&se);
        assert_eq!(rtn.unwrap(), "{}");

        // test for some value
        se.source_state_include.insert("aaa".to_string());
        let rtn = serde_json::to_string(&se).unwrap();
        assert_eq!(rtn, "{\"source_status_include\":[\"aaa\"]}");

        // deserialize

        let de: FlowSelector = serde_json::from_str(&rtn).unwrap();
        assert_eq!(de.context_exclude.is_empty(), true);
        assert_eq!(de.source_state_include.len(), 1);
    }
}
