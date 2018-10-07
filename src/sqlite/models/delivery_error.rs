use chrono::prelude::*;
use nature_common::*;
use sqlite::models::delivery::RawDelivery;
use super::super::schema::delivery_error;

#[derive(Debug)]
#[derive(Insertable)]
#[table_name = "delivery_error"]
pub struct RawDeliveryError {
    pub id: Vec<u8>,
    pub thing: String,
    pub data_type: i16,
    pub data: String,
    pub create_time: NaiveDateTime,
    pub msg: String,
}

impl RawDeliveryError {
    pub fn from_raw(err: &NatureError, raw: &RawDelivery) -> Self {
        RawDeliveryError {
            id: raw.id.clone(),
            thing: raw.thing.clone(),
            data_type: raw.data_type,
            data: raw.data.clone(),
            create_time: raw.create_time.clone(),
            msg: format!("{:?}", err),
        }
    }
}