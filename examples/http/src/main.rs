#![deny(
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(clippy::pedantic)]
#![allow(clippy::let_underscore_drop)]
#![forbid(unsafe_code)]

use domi::promise::Promise;
use domi::Context;
use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Replace this with any URL that accepts GET requests and returns text. Make sure they support CORS.
const API_URL: &str = "https://uselessfacts.jsph.pl/random.txt?language=en";

type Response = Result<String, String>;

enum State {
	Initial,
	Loading(Promise<Response>),
	Done(Response),
}

impl State {
	fn make_request(&mut self, context: Context) {
		*self = Self::Loading(Promise::spawn_async(make_request(), context));
	}
}

async fn make_request() -> Response {
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

fn main() {
	console_error_panic_hook::set_once();

	let root = web_sys::window()
		.unwrap()
		.document()
		.unwrap()
		.get_element_by_id("app")
		.unwrap();

	let mut state = State::Initial;
	domi::run(root.dyn_ref().unwrap(), move |mut ui| {
		if let State::Loading(response_promise) = &mut state {
			if let Some(response) = response_promise.try_take_by_ref() {
				state = State::Done(response);
			}
		}

		ui.element("header", "h1")
			.children()
			.text("HTTP request example");

		match &state {
			State::Initial => {
				let mut button = ui.element("make-req", "button");
				button.children().text("Make request");
				if button.clicked() {
					state.make_request(ui.context());
				}
			}
			State::Loading(..) => {
				ui.element("response", "p").children().text("Loading...");
			}
			State::Done(res) => {
				ui.element("response", "p")
					.children()
					.text(res.as_ref().map_or_else(
						|error| format!("Failure! An error occurred: {error:?}."),
						Clone::clone,
					));
				let mut again_button = ui.element("again", "button");
				again_button.children().text("Again!");
				if again_button.clicked() {
					state.make_request(ui.context());
				}
			}
		}
	});
}
