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

	// Our app state.
	let mut input = String::new();

	domi::run(root.dyn_into().unwrap(), move |mut ui| {
		ui.element("header", "h1").children().text("Rot13");

		ui.element("input-label", "label")
			.attr_static("for", "input")
			.children()
			.text("Input");

		// `input` will be automatically synchronized with the `<input>` element in the DOM.
		ui.text_input("input", &mut input)
			.attr_static("id", "input");

		ui.element("output-label", "label")
			.attr_static("for", "output")
			.children()
			.text("Output");

		ui.element("output", "p")
			.attr_static("id", "output")
			.children()
			.text(rot13(&input));
	});
}

fn rot13_char(ch: char) -> char {
	if ch.is_ascii_uppercase() {
		char::from(((u8::try_from(ch).unwrap() - b'A') + 13) % 26 + b'A')
	} else if ch.is_ascii_lowercase() {
		char::from(((u8::try_from(ch).unwrap() - b'a') + 13) % 26 + b'a')
	} else {
		ch
	}
}

fn rot13(input: &str) -> String {
	input.chars().map(rot13_char).collect()
}
