use crate::Result;
use crate::component::alert::unwrap_or_alert;
use crate::error::Error;
use crate::utils::get_window;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Headers, Request, RequestInit};

#[derive(Debug)]
pub struct Response {
    status: u16,
    body: Option<String>,
}

impl Response {
    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn body(&self) -> &Option<String> {
        &self.body
    }
}

/// A function to make simple AJAX requests.
pub async fn fetch(
    url: &str,
    method: &str,
    content_type: Option<&str>,
    body: Option<&str>,
) -> Result<Response> {
    let window = get_window()?;
    let request_init = RequestInit::new();
    if let Some(body) = body {
        request_init.set_body(&JsValue::from_str(body));
    }
    request_init.set_method(method);
    let headers = Headers::new()?;
    if let Some(content_type) = content_type {
        headers.append("Content-Type", content_type)?;
    }
    request_init.set_headers(&JsValue::from(&headers));
    let request =
        unwrap_or_alert(Request::new_with_str_and_init(url, &request_init).map_err(Error::from));
    let promise = window.fetch_with_request(&request);
    let response = wasm_bindgen_futures::JsFuture::from(promise)
        .await?
        .dyn_into::<web_sys::Response>()?;
    let status = response.status();
    Ok(Response {
        status,
        body: wasm_bindgen_futures::JsFuture::from(response.text()?)
            .await?
            .as_string(),
    })
}
