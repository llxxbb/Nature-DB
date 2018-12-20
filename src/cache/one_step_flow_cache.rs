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

type ITEM = (Option<Vec<OneStepFlow>>, Option<HashMap<Thing, Range<f32>>>);
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
                let label_groups = Self::get_label_groups(&relations);
                (Some(relations), Some(Self::weight_calculate(&label_groups)))
            }
            Err(err) => return Err(err)
        };
        let cpy = rtn.clone();
        cache.insert(from.clone(), rtn);
        Ok(cpy)
    }

    fn weight_filter(relations: &[OneStepFlow], balances: &HashMap<Thing, Range<f32>>) -> Vec<OneStepFlow> {
        let mut rtn: Vec<OneStepFlow> = Vec::new();
        let rnd = thread_rng().gen::<f32>();
        for m in relations {
            match balances.get(&m.to) {
                Some(rng) => if rng.contains(&rnd) {
                    rtn.push(m.clone());
                },
                None => rtn.push(m.clone())
            };
        }
        rtn
    }

    /// weight group will be cached
    fn weight_calculate(groups: &HashMap<String, Vec<OneStepFlow>>) -> HashMap<Thing, Range<f32>> {
        let mut rtn: HashMap<Thing, Range<f32>> = HashMap::new();
        // calculate "to `Thing`"'s weight
        for group in groups.values() {
            let sum = group.iter().fold(0u32, |sum, mapping| {
                sum + mapping.executor.weight.proportion
            });
            if sum == 0 {
                continue;
            }
            let mut begin = 0.0;
            let last = group.last().unwrap();
            for m in group {
                let w = m.executor.weight.proportion as f32 / sum as f32;
                let end = begin + w;
                if ptr::eq(m, last) {
                    // last must great 1
                    rtn.insert(m.to.clone(), begin..1.1);
                } else {
                    rtn.insert(m.to.clone(), begin..end);
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
