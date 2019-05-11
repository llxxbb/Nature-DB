use std::rc::Rc;
use crate::cache::ThingDefineCacheImpl;
use crate::service;
use crate::models::instance::InstanceServiceImpl;

pub struct DBService {
    pub thing_define: Rc<ThingDefineCacheImpl>,
    pub instance: Rc<InstanceServiceImpl>,
}

impl DBService {
    pub fn new() -> Self {
        let define = Rc::new(ThingDefineCacheImpl {});
        DBService {
            thing_define: define.clone(),
            instance: Rc::new(InstanceServiceImpl {}),
        }
    }
}

impl Default for service::DBService {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for DBService{}

lazy_static! {
    pub static ref SVC_DB : DBService = DBService::new();
}