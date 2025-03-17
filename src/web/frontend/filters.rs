use chrono::{NaiveDate, Utc};
use rocket::serde::json::Value;
use std::collections::HashMap;

pub fn is_in_the_past(date: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let date: NaiveDate = serde::Deserialize::deserialize(date)?;
    let now = Utc::now().date_naive();
    Ok(Value::Bool(date.cmp(&now).is_le()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;
    use rocket::serde::json::json;

    #[test]
    fn should_be_in_the_past() {
        let date = json!(Utc::now().date_naive().checked_sub_days(Days::new(1)));
        let result = is_in_the_past(&date, &HashMap::default()).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn should_not_be_in_the_past() {
        let date = json!(Utc::now().date_naive());
        let result = is_in_the_past(&date, &HashMap::default()).unwrap();
        assert!(result.as_bool().unwrap());
    }
}
