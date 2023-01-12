#![doc = include_str!("../README.md")]
#![deny(
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	missing_docs,
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
#![deny(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast as _;
use web_sys::HtmlElement;

use self::event::Event;
pub use self::vdom::{DomBuilder, ElementBuilder};

mod event;
mod id;
#[cfg(feature = "promise")]
pub mod promise;
pub mod vdom;

type RenderCallback = Box<dyn FnMut(DomBuilder<'_, '_>)>;

#[derive(Default)]
struct VDoms {
	last: vdom::VNodes,
	current: vdom::VNodes,
}

impl VDoms {
	fn advance(&mut self) {
		std::mem::swap(&mut self.last, &mut self.current);
		self.current.clear();
	}
}

struct Inner {
	// held only for ownership; never used
	event_handler: Option<Closure<dyn Fn(web_sys::Event)>>,
	vdoms: VDoms,

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
			vdoms: VDoms::default(),
			root,
			render,
		}
	}

	fn draw(&mut self, mode: DrawMode<'_>, backing: &Context) {
		self.vdoms.current.with_children_mut(|current_vdom| {
			let builder = match mode {
				DrawMode::ReactToEvent(event) => DomBuilder::new(None, Some(event), backing),
				DrawMode::BuildDom => DomBuilder::new(Some(current_vdom), None, backing),
			};
			(self.render)(builder);
		});

		if let DrawMode::BuildDom = mode {
			vdom::patch(
				&self.root,
				self.vdoms.last.children(),
				self.vdoms.current.children(),
			);

			self.vdoms.advance();
		}
	}
}

/// Your handle to `domi`.
///
/// Allows you to control the execution of the app.
#[derive(Clone)]
pub struct Context(Rc<RefCell<Inner>>);

impl Context {
	fn new(root: HtmlElement, render: RenderCallback) -> Self {
		let ret = Self(Rc::new(RefCell::new(Inner::new(root, render))));
		ret.register_js_event_handlers();
		ret
	}

	fn register_js_event_handlers(&self) {
		let event_handler = {
			let context = self.clone();
			move |event| context.js_event_handler(&event)
		};
		let event_handler = Closure::<dyn Fn(web_sys::Event)>::new(event_handler);
		let event_handler_js = event_handler.as_ref().unchecked_ref::<js_sys::Function>();

		let mut inner = self.0.borrow_mut();
		inner
			.root
			.add_event_listener_with_callback("click", event_handler_js)
			.unwrap();
		let replaced = inner.event_handler.replace(event_handler);
		debug_assert!(
			replaced.is_none(),
			"an event handler was already registered"
		);
	}

	fn unregister_js_event_handlers(&self) {
		let mut inner = self.0.borrow_mut();
		let event_handler = inner
			.event_handler
			.take()
			.expect("no event handler was registered");
		inner
			.root
			.remove_event_listener_with_callback(
				"click",
				event_handler.as_ref().unchecked_ref::<js_sys::Function>(),
			)
			.unwrap();
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

	/// Tell the app to update immediately.
	///
	/// Do not call this function while inside the `render` closure.
	/// Rather, register event handlers which call this function asynchronously.
	///
	/// As WASM is single-threaded, this will just update the app directly.
	pub fn request_update(&self) {
		self.build_dom();
	}

	/// Stop the app from rendering and reacting to events.
	pub fn stop(&self) {
		self.unregister_js_event_handlers();
	}
}

/// Set up and render the app, including responding to events.
///
/// This function returns after setting up the app, rather than blocking while running the UI. Subsequent updates occur through DOM event handlers.
pub fn run<F: FnMut(DomBuilder<'_, '_>) + 'static>(root: HtmlElement, render: F) {
	run_(root, Box::new(render));
}

fn run_(root: HtmlElement, render: Box<dyn FnMut(DomBuilder<'_, '_>)>) {
	let context = Context::new(root, render);
	context.build_dom();
}
