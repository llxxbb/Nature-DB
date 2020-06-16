use nature_common::TargetState;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RelationTarget {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub states: Option<TargetState>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub copy_para: Vec<u8>,
}