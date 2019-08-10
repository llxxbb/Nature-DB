use std::rc::Rc;
use crate::cache::MetaCacheImpl;
use crate::service;
use crate::models::instance::InstanceServiceImpl;

pub struct DBService {
    pub meta: Rc<MetaCacheImpl>,
    pub instance: Rc<InstanceServiceImpl>,
}

impl DBService {
    pub fn new() -> Self {
        let define = Rc::new(MetaCacheImpl {});
        DBService {
            meta: define.clone(),
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