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
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast as _;
use web_sys::HtmlElement;

use self::event::Event;
pub use self::vdom::DomBuilder;

mod event;
mod vdom;

#[derive(Default)]
struct State {
	// held only for ownership; never used
	draw_closure: Option<Closure<dyn Fn(web_sys::Event)>>,
	last_vdom: Vec<vdom::VNode>,
	current_vdom: Vec<vdom::VNode>,
}

/// This function returns after setting up the app, rather than blocking while running the UI.
/// Subsequent updates occur through DOM event handlers.
pub fn run<F: FnMut(DomBuilder<'_>) + 'static>(root: &HtmlElement, render: F) {
	run_(root, Box::new(render));
}

fn run_(root: &HtmlElement, render: Box<dyn FnMut(DomBuilder<'_>)>) {
	let draw = {
		let root = root.clone();
		let render = RefCell::new(render);
		move |event: Option<&Event>, state: &mut State| {
			let builder = DomBuilder::new(&mut state.current_vdom, event);
			(render.borrow_mut())(builder);

			vdom::patch_dom(&root, &state.last_vdom, &state.current_vdom);

			std::mem::swap(&mut state.last_vdom, &mut state.current_vdom);
			state.current_vdom.clear();
		}
	};

	let mut state = State::default();
	draw(None, &mut state);

	let state = Rc::new(RefCell::new(state));
	let draw = {
		let state = Rc::clone(&state);
		move |event: web_sys::Event| {
			if let Some(event) = Event::from_dom(&event) {
				let mut state = state.borrow_mut();
				draw(Some(&event), &mut state);
				// show any view changes due to events handled in the previous `draw` call
				draw(None, &mut state);
			}
		}
	};

	let draw_closure = Closure::<dyn Fn(web_sys::Event)>::new(draw);
	let draw_closure_js = draw_closure.as_ref().unchecked_ref();
	root
		.add_event_listener_with_callback("click", draw_closure_js)
		.unwrap();
	state.borrow_mut().draw_closure = Some(draw_closure);
}
