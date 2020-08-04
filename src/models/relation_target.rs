use nature_common::{is_default, TargetState};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RelationTarget {
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub states: Option<TargetState>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub append_para: Vec<u8>,
}