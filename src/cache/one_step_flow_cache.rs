extern crate rand;

use std::collections::HashMap;
use std::ops::Range;
use std::ptr;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::*;

use crate::{OneStepFlow, OneStepFlowDaoImpl};

use self::rand::{Rng, thread_rng};

/// all flows for one upper `Meta` and what a chance to lower `group`
type ITEM = (Option<Vec<OneStepFlow>>, Option<HashMap<Executor, Range<f32>>>);
type CACHE = Mutex<LruCache<Meta, ITEM>>;

lazy_static! {
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<Meta, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}


pub struct OneStepFlowCacheImpl;

type Dao = OneStepFlowDaoImpl;

impl OneStepFlowCacheImpl {
    pub fn get(from: &Meta) -> Result<Option<Vec<OneStepFlow>>> {
        let (relations, balances) = Self::get_balanced(from)?;
        if relations.is_none() {
            debug!("No relations of `Meta`: {:?}", from.get_full_key());
            Ok(None)
        } else {
            let vec = Self::weight_filter(&relations.unwrap(), &balances.unwrap());
            debug!("Available relations of `Meta`: {:?}， number : {:?}", from.get_full_key(), vec.len());
            Ok(Some(vec))
        }
    }
    fn get_balanced(from: &Meta) -> Result<ITEM> {
        let mut cache = CACHE_MAPPING.lock().unwrap();
        if let Some(balances) = cache.get(from) {
            return Ok(balances.clone());
        }
        let rtn = match Dao::get_relations(from) {
            Ok(None) => {
                (None, None)
            }
            Ok(Some(relations)) => {
                debug!("Get relations of `Meta`: {:?}， number : {:?}", from.get_full_key(), relations.len());
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
                sum + mapping.executor.proportion
            });
            if sum == 0 {
                continue;
            }
            // give a certain range for every participants
            let mut begin = 0.0;
            let last = group.last().unwrap();
            for m in group {
                let w = m.executor.proportion as f32 / sum as f32;
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
            let mappings = labels.entry(mapping.executor.group.clone()).or_insert_with(Vec::new);
            mappings.push(mapping.clone());
        }
        labels
    }
}

#[cfg(test)]
mod test_none_or_error {
    // TODO
//    use mockers::matchers::check;
//
//    use crate::test_util::*;
//
//    use super::*;
//
//    /// test cache also
//    #[test]
//    fn get_error() {
//        let from = Meta::new("error").unwrap();
//        let from_clone = from.clone();
//        let mocks = MyMocks::new();
//        mocks.s.expect(mocks.d_one_step.get_relations_call(check(move |t: &&Meta| t == &&from_clone)).and_return_clone(Err(NatureError::DaoEnvironmentError("can't connect".to_string()))).times(2));
//        let mocker = OneStepFlowCacheImpl {
//            dao: mocks.d_one_step.clone()
//        };
//
//        // this will call mocker
//        let result = mocker.get(&from);
//        assert_eq!(result, Err(NatureError::DaoEnvironmentError("can't connect".to_string())));
//        // and the repeated call will not call mocker but get from cache
//        let result = mocker.get(&from);
//        assert_eq!(result, Err(NatureError::DaoEnvironmentError("can't connect".to_string())));
//    }
//
//    /// test cache also
//    #[test]
//    fn get_none() {
//        let from = Meta::new("none").unwrap();
//        let from_clone = from.clone();
//        let mocks = MyMocks::new();
//        mocks.s.expect(mocks.d_one_step.get_relations_call(check(move |t: &&Meta| t == &&from_clone)).and_return_clone(Ok(None)).times(1));
//        let mocker = OneStepFlowCacheImpl {
//            dao: mocks.d_one_step.clone()
//        };
//
//        // this will call mocker
//        let result = mocker.get(&from);
//        assert_eq!(result.is_ok(), true);
//        let result = result.unwrap();
//        assert_eq!(result, None);
//        // and the repeated call will not call mocker but get from cache
//        let result = mocker.get(&from);
//        assert_eq!(result.is_ok(), true);
//        let result = result.unwrap();
//        assert_eq!(result, None);
//    }
}

/// There is one case will not to be tested : same target, different group.
/// This case will violate a principle: one source just has one executor only.
#[cfg(test)]
mod test_group_and_weight {
    // TODO
//    use mockers::matchers::check;
//
//    use crate::models::converter_cfg::OneStepFlow;
//    use crate::test_util::*;
//
//    use super::*;
//
//    #[test]
//    fn only_one_group_for_a_given_target() {
//        let _ = setup_logger();
//        let from = Meta::new("only_one_group_for_a_given_target").unwrap();
//
//        let relations = Ok(Some(vec![
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("oneFrom", "oneTo", "exe_0", "one", 10).unwrap(),
//        ]));
//
//        let from_clone = from.clone();
//        let mocks = MyMocks::new();
//        mocks.s.expect(mocks.d_one_step.get_relations_call(check(move |t: &&Meta| t == &&from_clone)).and_return_clone(relations).times(1));
//        let mocker = OneStepFlowCacheImpl {
//            dao: mocks.d_one_step.clone()
//        };
//
//        // this will call mocker
//        let result = mocker.get(&from);
//        let result = result.unwrap().unwrap();
//        assert_eq!(result.len(), 1);
//        // and the repeated call will not call mocker but get from cache
//        let result = mocker.get(&from);
//        let result = result.unwrap().unwrap();
//        assert_eq!(result.len(), 1);
//    }
//
//    #[test]
//    fn same_group_different_target() {
//        let from = Meta::new("same_group_different_target").unwrap();
//
//        let relations = Ok(Some(vec![
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("diffTarget", "targetA", "exe_5", "sameGroup", 2).unwrap(),
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("diffTarget", "targetB", "exe_6", "sameGroup", 8).unwrap(),
//        ]));
//
//        let from_clone = from.clone();
//        let mocks = MyMocks::new();
//        mocks.s.expect(mocks.d_one_step.get_relations_call(check(move |t: &&Meta| t == &&from_clone)).and_return_clone(relations).times(1));
//        let mocker = OneStepFlowCacheImpl {
//            dao: mocks.d_one_step.clone()
//        };
//
//
//        // this will call mocker
//        let result = mocker.get(&from);
//        let result = result.unwrap().unwrap();
//        assert_eq!(result.len(), 1);
//        // and the repeated call will not call mocker but get from cache
//        let result = mocker.get(&from);
//        let result = result.unwrap().unwrap();
//        assert_eq!(result.len(), 1);
//    }
//
//    #[test]
//    fn same_target_same_group() {
//        let _ = setup_logger();
//        let from = Meta::new("same_target_same_group").unwrap();
//
//        let relations = Ok(Some(vec![
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("sameTarget", "sameGroup", "exe_3", "same_group", 5).unwrap(),
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("sameTarget", "sameGroup", "exe_4", "same_group", 10).unwrap(),
//        ]));
//
//        let from_clone = from.clone();
//        let mocks = MyMocks::new();
//        mocks.s.expect(mocks.d_one_step.get_relations_call(check(move |t: &&Meta| t == &&from_clone)).and_return_clone(relations).times(1));
//        let mocker = OneStepFlowCacheImpl {
//            dao: mocks.d_one_step.clone()
//        };
//
//        // this will call mocker
//        let result = mocker.get(&from);
//        let result = result.unwrap().unwrap();
//        assert_eq!(result.len(), 1);
//        // and the repeated call will not call mocker but get from cache
//        let result = mocker.get(&from);
//        let result = result.unwrap().unwrap();
//        assert_eq!(result.len(), 1);
//    }
//
//    #[test]
//    fn weight_test() {
//        let _ = setup_logger();
//        let from = Meta::new("weight_test").unwrap();
//
//        let relations = Ok(Some(vec![
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("weight_from", "to_1", "exe_1", "grp", 2).unwrap(),
//            OneStepFlow::new_for_local_executor_with_group_and_proportion("weight_from", "to_2", "exe_2", "grp", 4).unwrap(),
//        ]));
//
//        let from_clone = from.clone();
//        let mocks = MyMocks::new();
//        mocks.s.expect(mocks.d_one_step.get_relations_call(check(move |t: &&Meta| t == &&from_clone)).and_return_clone(relations).times(1));
//        let mocker = OneStepFlowCacheImpl {
//            dao: mocks.d_one_step.clone()
//        };
//
//        let mut exe_1_cnt = 0;
//        let mut exe_2_cnt = 0;
//
//        for _i in 0..10 {
//            let result = mocker.get(&from);
//            let result = &result.unwrap().unwrap()[0];
//            match result.to.get_full_key().as_ref() {
//                "/B/to_1" => {
//                    exe_1_cnt = exe_1_cnt + 1;
//                }
//                "/B/to_2" => {
//                    exe_2_cnt = exe_2_cnt + 1;
//                }
//                _ => ()
//            }
//        }
//        let rate: f32 = exe_1_cnt as f32 / exe_2_cnt as f32;
//        println!("the rate is {}", rate);
//        assert_eq!(rate > 0.1 && rate < 0.4, true);
//    }
}


#[cfg(test)]
mod selector_test {
    // TODO
//    use super::*;
//
//    #[test]
//    fn source_status_needed() {
//        let mut set = HashSet::<String>::new();
//        set.insert("one".to_string());
//        set.insert("two".to_string());
//
//        let mut instance = Instance::default();
//
//        // set status required.
//        let osf = vec![OneStepFlow::new_for_source_status_needed("from", "to", &set).unwrap()];
//
//        // condition does not satisfy.
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.status = HashSet::new();
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.status.insert("three".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.status.insert("one".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//
//        // condition satisfy
//        instance.data.status.insert("two".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//        instance.data.status.insert("four".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//    }
//
//    #[test]
//    fn source_status_exclude() {
//        let mut set = HashSet::<String>::new();
//        set.insert("one".to_string());
//        set.insert("two".to_string());
//
//        let mut instance = Instance::default();
//
//        // set status required.
//        let osf = vec![OneStepFlow::new_for_source_status_excluded("from", "to", &set).unwrap()];
//
//        // condition satisfy
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//        instance.data.status = HashSet::new();
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//        instance.data.status.insert("three".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//
//        // condition does not satisfy
//        instance.data.status.insert("one".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.status.insert("two".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.status.remove("one");
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//    }
//
//    #[test]
//    fn context_needed() {
//        let mut set = HashSet::<String>::new();
//        set.insert("one".to_string());
//        set.insert("two".to_string());
//
//        let mut instance = Instance::default();
//
//        // set status required.
//        let osf = vec![OneStepFlow::new_for_context_include("from", "to", &set).unwrap()];
//
//        // condition does not satisfy.
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.context = HashMap::new();
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.context.insert("three".to_string(), "three".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.context.insert("one".to_string(), "one".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//
//        // condition satisfy
//        instance.data.context.insert("two".to_string(), "two".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//        instance.data.context.insert("four".to_string(), "four".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//    }
//
//    #[test]
//    fn context_exclude_test() {
//        let mut set = HashSet::<String>::new();
//        set.insert("one".to_string());
//        set.insert("two".to_string());
//
//        let mut instance = Instance::default();
//
//        // set status required.
//        let osf = vec![OneStepFlow::new_for_context_excluded("from", "to", &set).unwrap()];
//
//        // condition satisfy
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//        instance.data.context = HashMap::new();
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//        instance.data.context.insert("three".to_string(), "three".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_some(), true);
//
//        // condition does not satisfy
//        instance.data.context.insert("one".to_string(), "one".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.context.insert("two".to_string(), "two".to_string());
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//        instance.data.context.remove("one");
//        let option = RouteServiceImpl::filter_relations(&instance, osf.clone());
//        assert_eq!(option.is_none(), true);
//    }
}

#[cfg(test)]
mod other_test {
    // TODO
//    use super::*;
//
//    #[test]
//    fn input_cfg_is_empty() {
//        let instance = Instance::default();
//        let osf: Vec<OneStepFlow> = Vec::new();
//        let option = Mission::filter_relations(&instance, osf);
//        assert_eq!(option.is_none(), true)
//    }
//
//    #[test]
//    fn no_selector_but_only_executor() {
//        let instance = Instance::default();
//        let osf = vec![OneStepFlow::new_for_local_executor("from", "to", "local").unwrap()];
//        let option = Mission::filter_relations(&instance, osf);
//        assert_eq!(option.unwrap().len(), 1)
//    }
}

