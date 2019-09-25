extern crate rand;

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::*;

use crate::{MetaCacheGetter, MetaGetter, OneStepFlow, RelationGetter, RelationResult};

/// all flows for one upper `Meta` and what a chance to lower `group`
type ITEM = (Option<Vec<OneStepFlow>>, Option<HashMap<Executor, Range<f32>>>);
type CACHE = Mutex<LruCache<Meta, ITEM>>;
lazy_static! {
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<Meta, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type RelationCacheGetter = fn(&Meta, RelationGetter, MetaCacheGetter, MetaGetter) -> RelationResult;

pub struct OneStepFlowCacheImpl;

impl OneStepFlowCacheImpl {
    pub fn get(from: &Meta, getter: RelationGetter, meta_cache: MetaCacheGetter, meta: MetaGetter) -> RelationResult {
        let (relations, balances) = Self::get_balanced(from, getter, meta_cache, meta)?;
        if relations.is_none() {
            debug!("No relations of `Meta`: {:?}", from.get_full_key());
            Ok(None)
        } else {
            let vec = OneStepFlow::weight_filter(&relations.unwrap(), &balances.unwrap());
            debug!("Available relations of `Meta`: {:?}， number : {:?}", from.get_full_key(), vec.len());
            Ok(Some(vec))
        }
    }
    fn get_balanced(from: &Meta, getter: RelationGetter, meta_cache: MetaCacheGetter, meta: MetaGetter) -> Result<ITEM> {
        let mut cache = CACHE_MAPPING.lock().unwrap();
        if let Some(balances) = cache.get(from) {
            return Ok(balances.clone());
        }
        let rtn = match getter(from, meta_cache, meta) {
            Ok(None) => {
                (None, None)
            }
            Ok(Some(relations)) => {
                debug!("Get relations of `Meta`: {:?}， number : {:?}", from.get_full_key(), relations.len());
                let label_groups = OneStepFlow::get_label_groups(&relations);
                let weight_cal = OneStepFlow::weight_calculate(&label_groups);
                (Some(relations), Some(weight_cal))
            }
            Err(err) => return Err(err)
        };
        let cpy = rtn.clone();
        cache.insert(from.clone(), rtn);
        Ok(cpy)
    }
}

#[cfg(test)]
mod test {
    use nature_common::setup_logger;

    use crate::RawMeta;

    use super::*;

    fn rtn_err(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
        Err(NatureError::DaoEnvironmentError("can't connect".to_string()))
    }

    fn rtn_err2(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
        Err(NatureError::DaoEnvironmentError("another error".to_string()))
    }

    fn rtn_none(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
        Ok(None)
    }

    fn meta_cache(_: &mut Meta, _: MetaGetter) -> Result<()> {
        Ok(())
    }

    fn meta(m: &Meta) -> Result<Option<RawMeta>> {
        Ok(Some(RawMeta::from(m.clone())))
    }

    mod test_none_or_error {
        use super::*;

        #[test]
        fn get_error() {
            let from = Meta::new("error").unwrap();

            // this will call mocker
            let result = OneStepFlowCacheImpl::get(&from, rtn_err, meta_cache, meta);
            assert_eq!(result, Err(NatureError::DaoEnvironmentError("can't connect".to_string())));
            // error can't be catched
            let result = OneStepFlowCacheImpl::get(&from, rtn_err2, meta_cache, meta);
            assert_eq!(result, Err(NatureError::DaoEnvironmentError("another error".to_string())));
        }

        /// test cache also
        #[test]
        fn get_none() {
            let from = Meta::new("none").unwrap();
            // this will call mocker
            let result = OneStepFlowCacheImpl::get(&from, rtn_none, meta_cache, meta);
            assert_eq!(result.is_ok(), true);
            let result = result.unwrap();
            assert_eq!(result, None);
            // and the repeated call will not call mocker but get from cache
            let result = OneStepFlowCacheImpl::get(&from, rtn_err, meta_cache, meta);
            assert_eq!(result.is_ok(), true);
            let result = result.unwrap();
            assert_eq!(result, None);
        }
    }

    /// There is one case can not to be tested : same target, different group.
    /// This case will violate a principle: one source just has one executor only.
    mod test_group_and_weight {
        use super::*;

        #[test]
        fn only_one_group_for_a_given_target() {
            let _ = setup_logger();
            let from = Meta::new("only_one_group_for_a_given_target").unwrap();

            fn rtn_one(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("oneFrom", "oneTo", "exe_0", "one", 10).unwrap(),
                ]))
            }

            // this will call mocker
            let result = OneStepFlowCacheImpl::get(&from, rtn_one, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
            // and the repeated call will not call mocker but get from cache
            let result = OneStepFlowCacheImpl::get(&from, rtn_none, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn same_group_different_target() {
            let from = Meta::new("same_group_different_target").unwrap();

            fn rtn_some(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("diffTarget", "targetA", "exe_5", "sameGroup", 2).unwrap(),
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("diffTarget", "targetB", "exe_6", "sameGroup", 8).unwrap(),
                ]))
            }

            // this will call mocker
            let result = OneStepFlowCacheImpl::get(&from, rtn_some, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
            // and the repeated call will not call mocker but get from cache
            let result = OneStepFlowCacheImpl::get(&from, rtn_none, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn same_target_same_group() {
            let _ = setup_logger();
            let from = Meta::new("same_target_same_group").unwrap();

            fn rtn_some(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("sameTarget", "sameGroup", "exe_3", "same_group", 5).unwrap(),
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("sameTarget", "sameGroup", "exe_4", "same_group", 10).unwrap(),
                ]))
            }

            // this will call mocker
            let result = OneStepFlowCacheImpl::get(&from, rtn_some, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
            // and the repeated call will not call mocker but get from cache
            let result = OneStepFlowCacheImpl::get(&from, rtn_none, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn weight_test() {
            let _ = setup_logger();
            let from = Meta::new("weight_test").unwrap();

            fn rtn_some(_: &Meta, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("weight_from", "to_1", "exe_1", "grp", 1).unwrap(),
                    OneStepFlow::new_for_local_executor_with_group_and_proportion("weight_from", "to_2", "exe_2", "grp", 9).unwrap(),
                ]))
            }

            let mut exe_1_cnt = 0;
            let mut exe_2_cnt = 0;

            for _i in 0..100 {
                let result = OneStepFlowCacheImpl::get(&from, rtn_some, meta_cache, meta);
                let result = &result.unwrap().unwrap()[0];
                match result.to.get_full_key().as_ref() {
                    "/B/to_1" => {
                        exe_1_cnt = exe_1_cnt + 1;
                    }
                    "/B/to_2" => {
                        exe_2_cnt = exe_2_cnt + 1;
                    }
                    _ => ()
                }
            }
            let rate: f32 = exe_1_cnt as f32 / exe_2_cnt as f32;
            println!("the rate is {}", rate);
            assert_eq!(rate < 0.2, true);
        }
    }
}
