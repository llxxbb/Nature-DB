use std::rc::Rc;

use nature_common::util::*;

use super::*;

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
    use std::collections::HashMap;
    use std::collections::HashSet;

    use mockers::matchers::check;
    use mockers::Scenario;

    use super::*;

    #[test]
    fn can_not_get_from_cache() {
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<ThingDefineCacheTrait>();
        let mut instance = Instance::default();
        instance.data.thing.key = "/err".to_string();
        let expected_instance = instance.clone();
        scenario.expect(cond.get_call(check(move |t: &&Thing| **t == expected_instance.thing)).and_return(Err(NatureError::VerifyError("test error".to_string()))));
        let testee = InstanceServiceImpl { define_cache: Rc::new(cond) };
        let result = testee.verify(&mut instance);
        assert!(result.is_err());
    }

    #[test]
    fn can_get_from_cache() {
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<ThingDefineCacheTrait>();
        let mut instance = Instance::default();
        instance.data.thing.key = "/ok".to_string();
        let expected_instance = instance.clone();
        let define = ThingDefine::default();
        scenario.expect(cond.get_call(check(move |t: &&Thing| **t == expected_instance.thing)).and_return(Ok(define)));
        let testee = InstanceServiceImpl { define_cache: Rc::new(cond) };
        let result = testee.verify(&mut instance);
        assert!(result.is_ok());
    }

    #[test]
    fn id_generate() {
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<ThingDefineCacheTrait>();
        let service = InstanceServiceImpl {
            define_cache: Rc::new(cond)
        };
        let mut instance = Instance {
            id: 0,
            data: InstanceNoID {
                thing: Thing { key: "hello".to_string(), version: 3, thing_type: ThingType::Business, is_null: false },
                event_time: 0,
                execute_time: 0,
                create_time: 0,
                content: String::new(),
                context: HashMap::new(),
                status: HashSet::new(),
                status_version: 0,
                from: None,
            },
        };
        service.id_generate_if_not_set(&mut instance).unwrap();
        println!("{:?}", instance.id);
        assert_eq!(instance.id, 336556392135652841283170827290494770821);
    }
}