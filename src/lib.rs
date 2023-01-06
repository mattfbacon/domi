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
#[cfg(feature = "promise")]
pub mod promise;
mod vdom;

type RenderCallback = Box<dyn FnMut(DomBuilder<'_>)>;

struct Inner {
	// held only for ownership; never used
	event_handler: Option<Closure<dyn Fn(web_sys::Event)>>,
	last_vdom: Vec<vdom::VNode>,
	current_vdom: Vec<vdom::VNode>,

	root: HtmlElement,
	render: RenderCallback,
}

#[derive(Clone, Copy)]
enum DrawMode<'a> {
	ReactToEvent(&'a Event),
	BuildDom,
}

impl Inner {
	fn new(root: HtmlElement, render: RenderCallback) -> Self {
		Self {
			event_handler: None,
			last_vdom: Vec::new(),
			current_vdom: Vec::new(),
			root,
			render,
		}
	}

	fn draw(&mut self, mode: DrawMode<'_>, backing: &Context) {
		let builder = match mode {
			DrawMode::ReactToEvent(event) => DomBuilder::new(None, Some(event), backing),
			DrawMode::BuildDom => DomBuilder::new(Some(&mut self.current_vdom), None, backing),
		};
		(self.render)(builder);

		if let DrawMode::BuildDom = mode {
			vdom::patch_dom(&self.root, &self.last_vdom, &self.current_vdom);

			std::mem::swap(&mut self.last_vdom, &mut self.current_vdom);
			self.current_vdom.clear();
		}
	}
}

#[derive(Clone)]
pub struct Context(Rc<RefCell<Inner>>);

impl Context {
	fn new(root: &HtmlElement, render: RenderCallback) -> Self {
		let ret = Self(Rc::new(RefCell::new(Inner::new(root.clone(), render))));

		let event_handler = {
			let context = ret.clone();
			move |event| context.js_event_handler(&event)
		};
		let event_handler = Closure::<dyn Fn(web_sys::Event)>::new(event_handler);
		let event_handler_js = event_handler.as_ref().unchecked_ref();
		root
			.add_event_listener_with_callback("click", event_handler_js)
			.unwrap();
		ret.0.borrow_mut().event_handler = Some(event_handler);

		ret
	}

	fn draw(&self, mode: DrawMode<'_>) {
		self.0.borrow_mut().draw(mode, self);
	}

	fn react_to_event(&self, event: &Event) {
		self.draw(DrawMode::ReactToEvent(event));
	}

	fn build_dom(&self) {
		self.draw(DrawMode::BuildDom);
	}

	fn js_event_handler(&self, event: &web_sys::Event) {
		if let Some(event) = Event::from_dom(event) {
			self.react_to_event(&event);
			// show any view changes due to events handled in the previous `draw` call
			self.build_dom();
		}
	}

	pub fn request_update(&self) {
		self.build_dom();
	}
}

/// This function returns after setting up the app, rather than blocking while running the UI.
/// Subsequent updates occur through DOM event handlers.
pub fn run<F: FnMut(DomBuilder<'_>) + 'static>(root: &HtmlElement, render: F) {
	run_(root, Box::new(render));
}

fn run_(root: &HtmlElement, render: Box<dyn FnMut(DomBuilder<'_>)>) {
	let context = Context::new(root, render);
	context.build_dom();
}
