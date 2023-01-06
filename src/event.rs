use std::hash::Hash;

use wasm_bindgen::JsCast as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Id(u64);

impl std::fmt::Display for Id {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(formatter)
	}
}

impl Id {
	pub(crate) fn new(source: impl Hash) -> Self {
		Self(fxhash::hash64(&source))
	}

	pub(crate) fn with(self, source: impl Hash) -> Self {
		Self(fxhash::hash64(&(self.0, source)))
	}
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EventKind {
	Click,
}

impl EventKind {
	fn from_dom(dom: &str) -> Option<Self> {
		match dom {
			"click" => Some(Self::Click),
			_ => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Event {
	pub(crate) target: Id,
	pub(crate) kind: EventKind,
}

pub(crate) const ID_DATA_KEY: &str = "__domi_id";

impl Event {
	pub(crate) fn from_dom(dom: &web_sys::Event) -> Option<Self> {
		let kind = EventKind::from_dom(&dom.type_())?;
		let target = dom
			.target()
			.unwrap()
			.dyn_into::<web_sys::HtmlElement>()
			.unwrap()
			.dataset()
			.get(ID_DATA_KEY)?
			.parse()
			.unwrap();
		Some(Self {
			target: Id(target),
			kind,
		})
	}
}
