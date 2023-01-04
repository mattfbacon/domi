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
	console_error_panic_hook::set_once();

	let root = web_sys::window()
		.unwrap()
		.document()
		.unwrap()
		.get_element_by_id("app")
		.unwrap();

	let mut counter = 0;
	domi::run(root.dyn_ref().unwrap(), move |mut ui| {
		ui.element("label", "p")
			.children()
			.text(format!("The value is {counter}"));
		let mut button = ui.element("button", "button");
		button.children().text("Add one");
		if button.clicked() {
			counter += 1;
		}
	});
}
