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

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::{JsCast as _, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};

/// Replace this with any URL that accepts GET requests and returns text. Make sure they support CORS.
const API_URL: &str = "https://uselessfacts.jsph.pl/random.txt?language=en";

type Response = Result<String, String>;

enum State {
	Initial,
	Loading(Rc<RefCell<Option<Response>>>),
	Done(Response),
}

impl State {
	fn make_request(&mut self, updater: impl FnOnce() + 'static) {
		let response_place = Rc::new(RefCell::new(None));
		spawn_local(make_request(Rc::downgrade(&response_place), updater));
		*self = State::Loading(response_place);
	}
}

async fn make_request(response_place: Weak<RefCell<Option<Response>>>, updater: impl FnOnce()) {
	async fn helper() -> Result<String, JsValue> {
		let fetch_future: JsFuture = web_sys::window().unwrap().fetch_with_str(API_URL).into();
		let response = fetch_future.await?;

		let response: web_sys::Response = response.dyn_into().unwrap();
		if !response.ok() {
			return Err(format!("bad response status: {}", response.status_text()).into());
		}
		let body_future: JsFuture = response.text().unwrap().into();
		let body: String = body_future
			.await?
			.dyn_into::<js_sys::JsString>()
			.unwrap()
			.into();

		Ok(body)
	}

	let res = helper().await;
	if let Some(response_place) = response_place.upgrade() {
		*response_place.borrow_mut() = Some(res.map_err(|error| {
			error
				.dyn_into::<js_sys::Object>()
				.unwrap()
				.to_string()
				.into()
		}));
		updater();
	}
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
		if let State::Loading(response_place) = &state {
			let response = {
				let mut response_place = response_place.borrow_mut();
				response_place.take()
			};
			if let Some(response) = response {
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
					state.make_request(ui.updater());
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
					state.make_request(ui.updater());
				}
			}
		}
	});
}
