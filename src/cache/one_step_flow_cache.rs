extern crate rand;

use std::collections::HashMap;
use std::ops::Range;
use std::ptr;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use converter_cfg::OneStepFlow;
use define::OneStepFlowDaoTrait;
use nature_common::*;

use self::rand::{Rng, thread_rng};

/// all flows for one upper `Thing` and what a chance to lower `group`
type ITEM = (Option<Vec<OneStepFlow>>, Option<HashMap<Executor, Range<f32>>>);
type CACHE = Mutex<LruCache<Thing, ITEM>>;

lazy_static! {
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<Thing, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}

pub trait OneStepFlowCacheTrait {
    fn get(&self, from: &Thing) -> Result<Option<Vec<OneStepFlow>>>;
}

pub struct OneStepFlowCacheImpl {
    pub dao: Rc<OneStepFlowDaoTrait>
}

impl OneStepFlowCacheTrait for OneStepFlowCacheImpl {
    fn get(&self, from: &Thing) -> Result<Option<Vec<OneStepFlow>>> {
        let (relations, balances) = self.get_balanced(from)?;
        if relations.is_none() {
            debug!("no route info for : {:?}", from);
            Ok(None)
        } else {
            Ok(Some(Self::weight_filter(&relations.unwrap(), &balances.unwrap())))
        }
    }
}

impl OneStepFlowCacheImpl {
    fn get_balanced(&self, from: &Thing) -> Result<ITEM> {
        let mut cache = CACHE_MAPPING.lock().unwrap();
        if let Some(balances) = cache.get(from) {
            debug!("get balances from cache for thing : {:?}", from);
            return Ok(balances.clone());
        }
        let rtn = match self.dao.get_relations(from) {
            Ok(None) => {
                debug!("get none balances from db for thing : {:?}", from);
                (None, None)
            }
            Ok(Some(relations)) => {
                debug!("get relations for: {:?}， number : {:?}", from, relations.len());
                let label_groups = Self::get_label_groups(&relations);
                let weight_cal = Self::weight_calculate(&label_groups);
                (Some(relations), Some(weight_cal))
            }
            Err(err) => return Err(err)
        };
        let cpy = rtn.clone();
        cache.insert(from.clone(), rtn);
        Ok(cpy)
    }

    fn weight_filter(relations: &[OneStepFlow], balances: &HashMap<Executor, Range<f32>>) -> Vec<OneStepFlow> {
        let mut rtn: Vec<OneStepFlow> = Vec::new();
        let rnd = thread_rng().gen::<f32>();
        for m in relations {
            match balances.get(&m.executor) {
                Some(rng) => if rng.contains(&rnd) {
                    rtn.push(m.clone());
                },
                None => rtn.push(m.clone())
            };
        }
        rtn
    }

    /// weight group will be cached
    fn weight_calculate(groups: &HashMap<String, Vec<OneStepFlow>>) -> HashMap<Executor, Range<f32>> {
        let mut rtn: HashMap<Executor, Range<f32>> = HashMap::new();
        for group in groups.values() {
            // summarize the weight for one group
            let sum = group.iter().fold(0u32, |sum, mapping| {
                sum + mapping.executor.weight.proportion
            });
            if sum == 0 {
                continue;
            }
            // give a certain range for every participants
            let mut begin = 0.0;
            let last = group.last().unwrap();
            for m in group {
                let w = m.executor.weight.proportion as f32 / sum as f32;
                let end = begin + w;
                if ptr::eq(m, last) {
                    // last must great 1
                    rtn.insert(m.executor.clone(), begin..1.1);
                } else {
                    rtn.insert(m.executor.clone(), begin..end);
                }
                begin = end;
            }
        }
        rtn
    }

    /// group by labels. Only one flow will be used when there are same label. This can be used to switch two different flows smoothly.
    fn get_label_groups(maps: &[OneStepFlow]) -> HashMap<String, Vec<OneStepFlow>> {
        let mut labels: HashMap<String, Vec<OneStepFlow>> = HashMap::new();
        for mapping in maps {
            let mappings = labels.entry(mapping.executor.weight.group.clone()).or_insert_with(Vec::new);
            mappings.push(mapping.clone());
        }
        labels
    }
}

#[cfg(test)]
mod test_none_or_error {
    use mockers::matchers::check;
    use mockers::Scenario;

    use super::*;

    #[test]
    fn get_error() {
        let from = Thing::new("error").unwrap();
        let from_clone = from.clone();
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<OneStepFlowDaoTrait>();
        scenario.expect(cond.get_relations_call(check(move |t: &&Thing| t == &&from_clone)).and_return_clone(Err(NatureError::DaoEnvironmentError("can't connect".to_string()))).times(2));
        let mocker = OneStepFlowCacheImpl {
            dao: Rc::new(cond)
        };

        // this will call mocker
        let result = mocker.get(&from);
        assert_eq!(result, Err(NatureError::DaoEnvironmentError("can't connect".to_string())));
        // and the repeated call will not call mocker but get from cache
        let result = mocker.get(&from);
        assert_eq!(result, Err(NatureError::DaoEnvironmentError("can't connect".to_string())));
    }

    #[test]
    fn get_none() {
        let from = Thing::new("none").unwrap();
        let from_clone = from.clone();
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<OneStepFlowDaoTrait>();
        scenario.expect(cond.get_relations_call(check(move |t: &&Thing| t == &&from_clone)).and_return_clone(Ok(None)).times(1));
        let mocker = OneStepFlowCacheImpl {
            dao: Rc::new(cond)
        };

        // this will call mocker
        let result = mocker.get(&from);
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        assert_eq!(result, None);
        // and the repeated call will not call mocker but get from cache
        let result = mocker.get(&from);
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap();
        assert_eq!(result, None);
    }
}

/// There is one case will ont be tested : same target, different group.
/// This case will violate a principle: one source generate one target.
/// it would generate many target data,
#[cfg(test)]
mod test_group_and_weight {
    use mockers::matchers::check;
    use mockers::Scenario;

    use super::*;

    #[test]
    fn only_one_group_for_a_given_target() {
        let _ = setup_logger();
        let from = Thing::new("only_one_group_for_a_given_target").unwrap();

        let relations = Ok(Some(vec![
            OneStepFlow::new_for_local_executor_with_group_and_proportion("oneFrom", "oneTo", "exe_0", "one", 10).unwrap(),
        ]));

        let from_clone = from.clone();
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<OneStepFlowDaoTrait>();
        scenario.expect(cond.get_relations_call(check(move |t: &&Thing| t == &&from_clone)).and_return_clone(relations).times(1));
        let mocker = OneStepFlowCacheImpl {
            dao: Rc::new(cond)
        };

        // this will call mocker
        let result = mocker.get(&from);
        let result = result.unwrap().unwrap();
        assert_eq!(result.len(), 1);
        // and the repeated call will not call mocker but get from cache
        let result = mocker.get(&from);
        let result = result.unwrap().unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn same_group_different_target() {
        let from = Thing::new("same_group_different_target").unwrap();

        let relations = Ok(Some(vec![
            OneStepFlow::new_for_local_executor_with_group_and_proportion("diffTarget", "targetA", "exe_5", "sameGroup", 2).unwrap(),
            OneStepFlow::new_for_local_executor_with_group_and_proportion("diffTarget", "targetB", "exe_6", "sameGroup", 8).unwrap(),
        ]));

        let from_clone = from.clone();
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<OneStepFlowDaoTrait>();
        scenario.expect(cond.get_relations_call(check(move |t: &&Thing| t == &&from_clone)).and_return_clone(relations).times(1));
        let mocker = OneStepFlowCacheImpl {
            dao: Rc::new(cond)
        };

        // this will call mocker
        let result = mocker.get(&from);
        let result = result.unwrap().unwrap();
        assert_eq!(result.len(), 1);
        // and the repeated call will not call mocker but get from cache
        let result = mocker.get(&from);
        let result = result.unwrap().unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn same_target_same_group() {
        let _ = setup_logger();
        let from = Thing::new("same_target_same_group").unwrap();

        let relations = Ok(Some(vec![
            OneStepFlow::new_for_local_executor_with_group_and_proportion("sameTarget", "sameGroup", "exe_3", "same_group", 5).unwrap(),
            OneStepFlow::new_for_local_executor_with_group_and_proportion("sameTarget", "sameGroup", "exe_4", "same_group", 10).unwrap(),
        ]));

        let from_clone = from.clone();
        let scenario = Scenario::new();
        let cond = scenario.create_mock_for::<OneStepFlowDaoTrait>();
        scenario.expect(cond.get_relations_call(check(move |t: &&Thing| t == &&from_clone)).and_return_clone(relations).times(1));
        let mocker = OneStepFlowCacheImpl {
            dao: Rc::new(cond)
        };

        // this will call mocker
        let result = mocker.get(&from);
        let result = result.unwrap().unwrap();
        assert_eq!(result.len(), 1);
        // and the repeated call will not call mocker but get from cache
        let result = mocker.get(&from);
        let result = result.unwrap().unwrap();
        assert_eq!(result.len(), 1);
    }
}