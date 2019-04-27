use chrono::prelude::*;
use lazy_static::__Deref;
use serde_json;

use nature_common::*;

use crate::schema::plan;

#[derive(Debug)]
#[derive(Insertable, Queryable)]
#[table_name = "plan"]
pub struct RawPlanInfo {
    pub upstream: String,
    pub to_biz: String,
    pub to_version: i32,
    pub content: String,
    create_time: NaiveDateTime,
}

impl RawPlanInfo {
    pub fn new(plan: &PlanInfo) -> Result<RawPlanInfo> {
        let upstream = format!("{}:{}:{}:{}", plan.from_thing.get_full_key(), plan.from_thing.version, plan.from_sn, plan.from_sta_ver);
        Ok(RawPlanInfo {
            upstream,
            to_biz: plan.to.get_full_key(),
            to_version: plan.to.version,
            content: {
                let json = serde_json::to_string(&plan.plan)?;
                if json.len() > *PLAN_CONTENT_MAX_LENGTH.deref() {
                    return Err(NatureError::DaoLogicalError("content's length can' be over : ".to_owned() + &PLAN_CONTENT_MAX_LENGTH.to_string()));
                }
                json
            },
            create_time: Local::now().naive_local(),
        })
    }
    pub fn to_plan_info(&self) -> Result<PlanInfo> {
        let x: Vec<&str> = self.upstream.split(':').collect();
        if x.len() != 4 {
            return Err(NatureError::VerifyError("format error : ".to_owned() + &self.upstream));
        }
        Ok(PlanInfo {
            from_thing: Thing::from_full_key(x[0], x[1].parse()?)?,
            from_sn: x[2].parse()?,
            from_sta_ver: x[3].parse()?,
            to: Thing::from_full_key(&self.to_biz,self.to_version)?,
            plan: serde_json::from_str(&self.content)?,
        })
    }
}

