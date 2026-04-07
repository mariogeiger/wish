use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};

async fn fetch_json<T: DeserializeOwned>(method: &str, url: &str, body: Option<String>) -> Result<T, String> {
    let opts = RequestInit::new();
    opts.set_method(method);

    let headers = Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);

    if let Some(b) = body {
        opts.set_body(&wasm_bindgen::JsValue::from_str(&b));
    }

    let request = Request::new_with_str_and_init(url, &opts).map_err(|e| format!("{e:?}"))?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "not a Response")?;

    if !resp.ok() {
        let status = resp.status();
        let text = JsFuture::from(resp.text().map_err(|e| format!("{e:?}"))?)
            .await
            .map_err(|e| format!("{e:?}"))?
            .as_string()
            .unwrap_or_default();
        return Err(format!("HTTP {status}: {text}"));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;
    let result: T = serde_wasm_bindgen::from_value(json).map_err(|e| format!("parse: {e:?}"))?;
    Ok(result)
}

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    fetch_json("GET", url, None).await
}

pub async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, String> {
    let json = serde_json::to_string(body).map_err(|e| format!("{e}"))?;
    fetch_json("POST", url, Some(json)).await
}

pub async fn put<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, String> {
    let json = serde_json::to_string(body).map_err(|e| format!("{e}"))?;
    fetch_json("PUT", url, Some(json)).await
}
