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

use wasm_bindgen::JsCast as _;

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

	// This is the state of the app.
	// Notice how no explicit type declaration is necessary; we can simply move some variables into the closure and use them ad-hoc as the app's state.
	let mut counter = 0;

	// Actually start the app.
	// The first argument is the HTML element which will be root of the app.
	// The second argument is the render callback. `ui` allows building the DOM.
	domi::run(root.dyn_into().unwrap(), move |mut ui| {
		// Add an element.
		// The first argument is the ID. See the documentation at the crate root for more information about IDs.
		// The second argument is simply the tag name of the element.
		// This is a simple element with a single child so it can be declared in the following chaining form.
		ui.element("label", "p")
			// `children` lets you add children to the element.
			.children()
			// Add a simple text node.
			.text(format!("The value is {counter}"));

		// Add another element. In this case, we assign it to a `let`-binding in order to allow accessing the `clicked` method after adding children.
		let mut button = ui.element("button", "button");
		button.children().text("Add one");
		// Handle clicks on the button...
		if button.clicked() {
			// ...by updating the state. The app will then render again, and the updated count will show.
			counter += 1;
		}
	});
}
