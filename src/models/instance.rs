use nature_common::Instance;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DelayedInstances {
    pub carrier_id: Vec<u8>,
    pub result: CallbackResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CallbackResult {
    Err(String),
    Instances(Vec<Instance>),
}

impl Default for CallbackResult {
    fn default() -> Self {
        CallbackResult::Instances(Vec::new())
    }
}

pub struct InstanceServiceImpl;

unsafe impl Sync for InstanceServiceImpl {}
