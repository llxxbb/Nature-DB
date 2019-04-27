use std::rc::Rc;

use mockers::Scenario;
use mockers_derive::mock;
use crate::models::converter_cfg::OneStepFlow;
use crate::models::define::OneStepFlowDaoTrait;
use crate::models::thing_define::ThingDefine;
use crate::models::define::ThingDefineCacheTrait;
use crate::*;

mock! {
    OneStepFlowDaoTraitMock,
    self,
    trait OneStepFlowDaoTrait{
        fn get_relations(&self, from: &Thing) -> Result<Option<Vec<OneStepFlow>>>;
    }
}

mock! {
    ThingDefineCacheTraitMock,
    self,
    trait ThingDefineCacheTrait{
        fn get(&self, thing: &Thing) -> Result<ThingDefine>;
    }
}


pub struct MyMocks {
    pub s: Scenario,
    pub d_one_step: Rc<OneStepFlowDaoTraitMock>,
    pub c_thing_define: Rc<ThingDefineCacheTraitMock>,
}

impl MyMocks {
    pub fn new() -> MyMocks {
        let s = Scenario::new();
        let d_one_step = Rc::new(s.create_mock::<OneStepFlowDaoTraitMock>());
        let c_thing_define = Rc::new(s.create_mock::<ThingDefineCacheTraitMock>());
        MyMocks {
            s,
            d_one_step,
            c_thing_define,
        }
    }
}
