use chrono::{NaiveDate, Utc};
use serde_json::Value;
use std::collections::HashMap;

pub fn is_in_the_past(date: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let date: NaiveDate = serde::Deserialize::deserialize(date)?;
    let now = Utc::now().date_naive();
    Ok(Value::Bool(date.cmp(&now).is_le()))
}
