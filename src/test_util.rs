use std::rc::Rc;

use mockers::Scenario;
use mockers_derive::mock;
use crate::models::define::ThingDefineCacheTrait;
use crate::*;


mock! {
    ThingDefineCacheTraitMock,
    self,
    trait ThingDefineCacheTrait{
        fn get(&self, thing: &Thing) -> Result<RawThingDefine>;
    }
}


pub struct MyMocks {
    pub s: Scenario,
    pub c_thing_define: Rc<ThingDefineCacheTraitMock>,
}

impl MyMocks {
    pub fn new() -> MyMocks {
        let s = Scenario::new();
        let c_thing_define = Rc::new(s.create_mock::<ThingDefineCacheTraitMock>());
        MyMocks {
            s,
            c_thing_define,
        }
    }
}
