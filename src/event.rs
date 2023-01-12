use wasm_bindgen::JsCast as _;

use crate::id::Id;

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

impl Event {
	pub(crate) fn from_dom(dom: &web_sys::Event) -> Option<Self> {
		let kind = EventKind::from_dom(&dom.type_())?;
		let target = dom
			.target()
			.unwrap()
			.dyn_into::<web_sys::HtmlElement>()
			.unwrap()
			.dataset()
			.get(Id::DATA_KEY)?
			.parse()
			.unwrap();
		Some(Self {
			target: Id(target),
			kind,
		})
	}
}
