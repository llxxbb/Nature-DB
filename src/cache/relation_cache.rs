extern crate rand;

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Mutex;
use std::time::Duration;

use lru_time_cache::LruCache;

use nature_common::*;

use crate::{MetaCacheGetter, MetaGetter, Relation, RelationGetter, RelationResult};

/// all flows for one upper `Meta` and what a chance to lower `group`
type ITEM = (Option<Vec<Relation>>, Option<HashMap<Executor, Range<f32>>>);
type CACHE = Mutex<LruCache<String, ITEM>>;
lazy_static! {
    static ref CACHE_MAPPING: CACHE = Mutex::new(LruCache::<String, ITEM>::with_expiry_duration(Duration::from_secs(3600)));
}

pub type RelationCacheGetter = fn(&str, RelationGetter, MetaCacheGetter, MetaGetter) -> RelationResult;

pub struct RelationCacheImpl;

impl RelationCacheImpl {
    pub fn get(meta_from: &str, getter: RelationGetter, meta_cache: MetaCacheGetter, meta: MetaGetter) -> RelationResult {
        let (relations, balances) = Self::get_balanced(meta_from, getter, meta_cache, meta)?;
        if relations.is_none() {
            debug!("No relations of `Meta`: {}", meta_from);
            Ok(None)
        } else {
            let vec = Relation::weight_filter(&relations.unwrap(), &balances.unwrap());
            // for r in &vec {
            //     debug!("relation can be used for `Meta`: {}， relation : {}", meta_from, r.relation_string());
            // }
            Ok(Some(vec))
        }
    }
    fn get_balanced(meta_from: &str, getter: RelationGetter, meta_cache: MetaCacheGetter, meta: MetaGetter) -> Result<ITEM> {
        let mut cache = CACHE_MAPPING.lock().unwrap();
        if let Some(balances) = cache.get(meta_from) {
            return Ok(balances.clone());
        }
        let rtn = match getter(meta_from, meta_cache, meta) {
            Ok(None) => {
                (None, None)
            }
            Ok(Some(relations)) => {
                let label_groups = Relation::get_label_groups(&relations);
                let weight_cal = Relation::weight_calculate(&label_groups);
                (Some(relations), Some(weight_cal))
            }
            Err(err) => return Err(err)
        };
        let cpy = rtn.clone();
        cache.insert(meta_from.to_string(), rtn);
        Ok(cpy)
    }
}

#[cfg(test)]
mod test {
    use nature_common::setup_logger;

    use crate::RawMeta;

    use super::*;

    fn rtn_err(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
        Err(NatureError::EnvironmentError("can't connect".to_string()))
    }

    fn rtn_err2(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
        Err(NatureError::EnvironmentError("another error".to_string()))
    }

    fn rtn_none(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
        Ok(None)
    }

    fn meta_cache(meta_str: &str, _: MetaGetter) -> Result<Meta> {
        Ok(Meta::from_string(meta_str)?)
    }

    fn meta(m: &str) -> Result<Option<RawMeta>> {
        Ok(Some(RawMeta::from(Meta::from_string(m)?)))
    }

    mod test_none_or_error {
        use super::*;

        #[test]
        fn get_error() {
            let from = "B:error:1";

            // this will call mocker
            let result = RelationCacheImpl::get(&from, rtn_err, meta_cache, meta);
            assert_eq!(result, Err(NatureError::EnvironmentError("can't connect".to_string())));
            // error can't be catched
            let result = RelationCacheImpl::get(&from, rtn_err2, meta_cache, meta);
            assert_eq!(result, Err(NatureError::EnvironmentError("another error".to_string())));
        }

        /// test cache also
        #[test]
        fn get_none() {
            let from = "B:none:1";
            // this will call mocker
            let result = RelationCacheImpl::get(&from, rtn_none, meta_cache, meta);
            assert_eq!(result.is_ok(), true);
            let result = result.unwrap();
            assert_eq!(result, None);
            // and the repeated call will not call mocker but get from cache
            let result = RelationCacheImpl::get(&from, rtn_err, meta_cache, meta);
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
            let from = "B:only_one_group_for_a_given_target:1";

            fn rtn_one(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    new_for_local_executor_with_group_and_weight("B:oneFrom:1", "B:oneTo:1", "exe_0", "one", 10).unwrap(),
                ]))
            }

            // this will call mocker
            let result = RelationCacheImpl::get(&from, rtn_one, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
            // and the repeated call will not call mocker but get from cache
            let result = RelationCacheImpl::get(&from, rtn_none, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn same_group_different_target() {
            let from = "B:same_group_different_target:1";

            fn rtn_some(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    new_for_local_executor_with_group_and_weight("B:diffTarget:1", "B:targetA:1", "exe_5", "sameGroup", 2).unwrap(),
                    new_for_local_executor_with_group_and_weight("B:diffTarget:1", "B:targetB:1", "exe_6", "sameGroup", 8).unwrap(),
                ]))
            }

            // this will call mocker
            let result = RelationCacheImpl::get(&from, rtn_some, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
            // and the repeated call will not call mocker but get from cache
            let result = RelationCacheImpl::get(&from, rtn_none, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn same_target_same_group() {
            let _ = setup_logger();
            let from = "B:same_target_same_group:1";

            fn rtn_some(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    new_for_local_executor_with_group_and_weight("B:sameTarget:1", "B:sameGroup:1", "exe_3", "same_group", 5).unwrap(),
                    new_for_local_executor_with_group_and_weight("B:sameTarget:1", "B:sameGroup:1", "exe_4", "same_group", 10).unwrap(),
                ]))
            }

            // this will call mocker
            let result = RelationCacheImpl::get(&from, rtn_some, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
            // and the repeated call will not call mocker but get from cache
            let result = RelationCacheImpl::get(&from, rtn_none, meta_cache, meta);
            let result = result.unwrap().unwrap();
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn weight_test() {
            let _ = setup_logger();
            let from = "B:weight_test:1";

            fn rtn_some(_: &str, _: MetaCacheGetter, _: MetaGetter) -> RelationResult {
                Ok(Some(vec![
                    new_for_local_executor_with_group_and_weight("B:weight_from:1", "B:to_1:1", "exe_1", "grp", 1).unwrap(),
                    new_for_local_executor_with_group_and_weight("B:weight_from:1", "B:to_2:1", "exe_2", "grp", 9).unwrap(),
                ]))
            }

            let mut exe_1_cnt = 0;
            let mut exe_2_cnt = 0;

            for _i in 0..1000 {
                let result = RelationCacheImpl::get(&from, rtn_some, meta_cache, meta);
                let result = &result.unwrap().unwrap()[0];
                match result.to.meta_string().as_ref() {
                    "B:to_1:1" => {
                        exe_1_cnt = exe_1_cnt + 1;
                    }
                    "B:to_2:1" => {
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

    fn new_for_local_executor_with_group_and_weight(from: &str, to: &str, local_executor: &str, group: &str, weight: u32) -> Result<Relation> {
        Ok(Relation {
            from: from.to_string(),
            to: Meta::from_string(to)?,
            selector: None,
            executor: Executor {
                protocol: Protocol::LocalRust,
                url: local_executor.to_string(),
                group: group.to_string(),
                weight,
            },
            use_upstream_id: false,
            target_states: None,
            delay: 0,
        })
    }
}
