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
use domi::{Context, DomBuilder};
use wasm_bindgen::JsCast as _;

use self::request::{make_request, Response};

// Just an adapter for making an HTTP request with the Fetch API.
// Can be ignored for the purposes of this example, unless you're curious.
mod request;

// In this case, we've chosen to explicitly declare a state type rather than moving variables into the render closure ad-hoc.
//
// This is more common in larger apps where the structure is helpful.
// For one, it allows us to define a `ui` method on the type and refer to the state with `self`.
//
// By providing a `Default` implementation, we let `main` just use `State::default()` to initialize the app state.
#[derive(Default)]
enum State {
	#[default]
	Initial,
	Loading(Promise<Response>),
	Done(String),
}

impl State {
	// Make the request and switch to the loading state.
	fn make_request(&mut self, context: Context) {
		*self = Self::Loading(Promise::spawn_async(make_request(), context));
	}

	// Render the app.
	//
	// We implement this as a method of this type rather than writing the rendering code in the render closure passed to `domi::run`.
	// This allows the code to be a bit cleaner since we can refer to the state as `self`.
	fn render(&mut self, ui: &mut DomBuilder<'_, '_>) {
		// We make sure to handle a possible response from the request first, so that the UI will represent the updated state.
		// If we didn't do this here, and instead did it in the later `match` block, we would change to `State::Done` without rendering again, so the UI would continue to show the loading message.
		//
		// This pattern is common when handling promises. Notice specifically how we immediately replace the state with a new variant of the enum if we receive a response.
		// This is because `Promise::try_take` logically consumes the `Promise`, so we shouldn't keep it around after we "consume" it.
		if let Self::Loading(response_promise) = self {
			if let Some(response) = response_promise.try_take() {
				let message =
					response.unwrap_or_else(|error| format!("Failure! An error occurred: {error:?}."));
				*self = State::Done(message);
			}
		}

		ui.element("header", "h1")
			.children()
			.text("HTTP request example");

		// In an application with multiple states, this pattern with a `match` block is very common.
		match self {
			Self::Initial => {
				let mut button = ui.element("make-req", "button");
				button.children().text("Make request");
				if button.clicked() {
					self.make_request(ui.context().clone());
				}
			}
			Self::Loading(..) => {
				ui.element("response", "p").children().text("Loading...");
			}
			Self::Done(res) => {
				ui.element("response", "p").children().text(res);

				let mut again_button = ui.element("again", "button");
				again_button.children().text("Again!");
				if again_button.clicked() {
					self.make_request(ui.context().clone());
				}
			}
		}
	}
}

fn main() {
	// This makes panics show up in the browser console.
	console_error_panic_hook::set_once();

	// A common incantation to interface with JS and get the root element.
	let root = web_sys::window()
		.unwrap()
		.document()
		.unwrap()
		.get_element_by_id("app")
		.unwrap();

	// We use the `Default` implementation so we don't have to worry about the internals of `State` in `main`...
	let mut state = State::default();
	domi::run(root.dyn_into().unwrap(), move |mut ui| {
		// ...and likewise we use the `render` method here for the same reason.
		state.render(&mut ui);
	});
}
