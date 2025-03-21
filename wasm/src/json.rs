use crate::component::alert::{AlertLevel, create_alert};
use crate::error::DEFAULT_ERROR_MESSAGE;
use serde::{de, ser};

pub fn to_string<T>(value: &T) -> String
where
    T: ser::Serialize + ?Sized,
{
    match serde_json_wasm::to_string(value) {
        Ok(body) => body,
        Err(error) => {
            create_alert(DEFAULT_ERROR_MESSAGE, AlertLevel::Error);
            log::error!("{:#?}", error);
            panic!("{error:#?}");
        }
    }
}

pub fn from_str<T>(s: &str) -> T
where
    T: de::DeserializeOwned,
{
    match serde_json_wasm::from_str(s) {
        Ok(body) => body,
        Err(error) => {
            create_alert(DEFAULT_ERROR_MESSAGE, AlertLevel::Error);
            log::error!("{:#?}", error);
            panic!("{error:#?}");
        }
    }
}
