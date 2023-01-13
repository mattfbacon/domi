use wasm_bindgen::JsCast as _;

use crate::id::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventKind {
	Click,
	Change,
}

pub(crate) const HANDLED_EVENTS: &[(&str, EventKind)] =
	&[("click", EventKind::Click), ("change", EventKind::Change)];

impl EventKind {
	fn from_dom(dom: &str) -> Option<Self> {
		HANDLED_EVENTS
			.iter()
			.copied()
			.find(|&(name, _)| dom == name)
			.map(|(_, val)| val)
	}
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Event {
	pub(crate) target: web_sys::HtmlElement,
	pub(crate) target_id: Id,
	pub(crate) kind: EventKind,
}

impl Event {
	pub(crate) fn from_dom(dom: &web_sys::Event) -> Option<Self> {
		let kind = EventKind::from_dom(&dom.type_())?;
		let target = dom
			.target()
			.unwrap()
			.dyn_into::<web_sys::HtmlElement>()
			.unwrap();
		let target_id = target.dataset().get(Id::DATA_KEY)?.parse().unwrap();
		Some(Self {
			target,
			target_id: Id(target_id),
			kind,
		})
	}
}
