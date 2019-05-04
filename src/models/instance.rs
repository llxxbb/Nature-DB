use std::rc::Rc;

use nature_common::Instance;
use nature_common::Result;
use nature_common::util::*;

use crate::models::define::ThingDefineCacheTrait;

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

pub trait InstanceServiceTrait {
    fn verify(&self, instance: &mut Instance) -> Result<u128>;
    /// gegerate by Hash.
    fn id_generate_if_not_set(&self, instance: &mut Instance) -> Result<u128>;
}

pub struct InstanceServiceImpl {
    pub define_cache: Rc<ThingDefineCacheTrait>,
}

impl InstanceServiceTrait for InstanceServiceImpl {
    /// check key whether defined
    /// generate id by hashing if it is not set.
    fn verify(&self, instance: &mut Instance) -> Result<u128> {
        // just see whether it was configured.
        self.define_cache.get(&instance.data.thing)?;
        self.id_generate_if_not_set(instance)
    }
    fn id_generate_if_not_set(&self, instance: &mut Instance) -> Result<u128> {
        if instance.id == 0 {
            instance.id = generate_id(&instance.data)?;
        }
        Ok(instance.id)
    }
}

unsafe impl Sync for InstanceServiceImpl {}

#[cfg(test)]
mod test {
    use mockers::matchers::check;

    use nature_common::{NatureError, Thing};

    use crate::RawThingDefine;
    use crate::test_util::*;

    use super::*;

    #[test]
    fn can_not_get_from_cache() {
        let mocks = MyMocks::new();
        let mut instance = Instance::new("/err").unwrap();
        let expected_instance = instance.clone();
        mocks.s.expect(mocks.c_thing_define.get_call(check(move |t: &&Thing| **t == expected_instance.thing)).and_return(Err(NatureError::VerifyError("test error".to_string()))));
        let testee = InstanceServiceImpl { define_cache: mocks.c_thing_define.clone() };
        let result = testee.verify(&mut instance);
        assert!(result.is_err());
    }

    #[test]
    fn can_get_from_cache() {
        let mocks = MyMocks::new();
        let mut instance = Instance::new("/ok").unwrap();
        let expected_instance = instance.clone();
        let define = RawThingDefine::default();
        mocks.s.expect(mocks.c_thing_define.get_call(check(move |t: &&Thing| **t == expected_instance.thing)).and_return(Ok(define)));
        let testee = InstanceServiceImpl { define_cache: mocks.c_thing_define.clone() };
        let result = testee.verify(&mut instance);
        assert!(result.is_ok());
    }

    #[test]
    fn id_generate() {
        let mocks = MyMocks::new();
        let service = InstanceServiceImpl {
            define_cache: mocks.c_thing_define.clone()
        };
        let mut instance = Instance::new("hello").unwrap();
        service.id_generate_if_not_set(&mut instance).unwrap();
        println!("{:?}", instance.id);
        assert_eq!(instance.id, 336556392135652841283170827290494770821);
    }
}