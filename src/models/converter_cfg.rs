use std::collections::HashSet;

use nature_common::*;

use crate::OneStepFlow;

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteInfo {
    pub instance: Instance,
    pub maps: Vec<OneStepFlow>,
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
}
