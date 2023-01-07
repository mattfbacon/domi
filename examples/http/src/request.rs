#![allow(clippy::module_name_repetitions)]

use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Replace this with any URL that accepts GET requests and returns text. Make sure they support CORS.
const API_URL: &str = "https://uselessfacts.jsph.pl/random.txt?language=en";

pub type Response = Result<String, String>;

pub async fn make_request() -> Response {
	async fn helper() -> Result<String, JsValue> {
		let window = web_sys::window().unwrap();

		let response: web_sys::Response = JsFuture::from(window.fetch_with_str(API_URL))
			.await?
			.dyn_into()
			.unwrap();
		if !response.ok() {
			return Err(format!("bad response status: {}", response.status_text()).into());
		}

		let body: String = JsFuture::from(response.text().unwrap())
			.await?
			.dyn_into::<js_sys::JsString>()
			.unwrap()
			.into();

		Ok(body)
	}

	helper().await.map_err(|error| {
		error
			.dyn_into::<js_sys::Object>()
			.unwrap()
			.to_string()
			.into()
	})
}
