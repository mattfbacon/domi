#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::hash::Hash;

use crate::event::{Event, EventKind, Id};
use crate::vdom::{VNode, VNodeElement};

enum ElementOrId<'a> {
	Element(&'a mut VNodeElement),
	Id(Id),
}

impl ElementOrId<'_> {
	fn as_element(&mut self) -> Option<&mut VNodeElement> {
		match self {
			Self::Element(element) => Some(element),
			Self::Id(..) => None,
		}
	}

	fn id(&self) -> Id {
		match self {
			Self::Element(element) => element.id,
			Self::Id(id) => *id,
		}
	}
}

pub struct ElementBuilder<'a> {
	vdom: ElementOrId<'a>,
	event: Option<&'a Event>,
}

impl ElementBuilder<'_> {
	pub fn attr(mut self, attr: impl Into<String>, value: impl Into<String>) -> Self {
		if let Some(vdom) = self.vdom.as_element() {
			vdom.attributes.insert(attr.into(), value.into());
		}
		self
	}

	pub fn children(&mut self) -> DomBuilder<'_> {
		DomBuilder {
			parent_id: Some(self.vdom.id()),
			vdom: self.vdom.as_element().map(|element| &mut element.children),
			event: self.event,
		}
	}

	pub fn clicked(&self) -> bool {
		self
			.event
			.filter(|event| event.target == self.vdom.id() && event.kind == EventKind::Click)
			.is_some()
	}
}

pub struct DomBuilder<'a> {
	parent_id: Option<Id>,
	vdom: Option<&'a mut Vec<VNode>>,
	event: Option<&'a Event>,
}

impl<'a> DomBuilder<'a> {
	/// If `None` is provided for `vdom`, then don't actually build a DOM, but still process events.
	pub(crate) fn new(vdom: Option<&'a mut Vec<VNode>>, event: Option<&'a Event>) -> Self {
		Self {
			vdom,
			event,
			parent_id: None,
		}
	}

	pub fn text(&mut self, text: impl Into<String>) {
		if let Some(vdom) = &mut self.vdom {
			vdom.push(VNode::Text(text.into()));
		}
	}

	fn element_(&mut self, id: Id, tag: String) -> ElementBuilder<'_> {
		if let Some(vdom) = &mut self.vdom {
			let idx = vdom.len();
			vdom.push(VNode::Element(VNodeElement {
				id,
				tag,
				attributes: HashMap::new(),
				children: Vec::new(),
			}));
			let VNode::Element(element) = &mut vdom[idx] else { unreachable!() };
			ElementBuilder {
				vdom: ElementOrId::Element(element),
				event: self.event,
			}
		} else {
			ElementBuilder {
				vdom: ElementOrId::Id(id),
				event: self.event,
			}
		}
	}

	#[inline]
	pub fn element(&mut self, id: impl Hash, tag: impl Into<String>) -> ElementBuilder<'_> {
		self.element_(
			self
				.parent_id
				.map_or_else(|| Id::new(&id), |parent_id| parent_id.with(&id)),
			tag.into(),
		)
	}
}

macro_rules! element_methods {
	($($element:ident),* $(,)?) => {
		/// Shorter constructors for elements
		impl DomBuilder<'_> {
			$(
				#[inline]
				pub fn $element(&mut self, id: impl Hash) -> ElementBuilder<'_> {
					self.element(id, stringify!($element))
				}
			)*
		}
	}
}

element_methods![
	a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, br, button, canvas,
	caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, div, dl, dt,
	em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, header, hgroup,
	hr, i, iframe, img, input, ins, kbd, label, legent, li, link, main, map, mark, menu, meter, nav,
	noscript, object, ol, optgroup, option, output, p, picture, pre, progress, q, rp, rt, ruby, s,
	samp, section, select, slot, small, source, span, strong, sub, summary, sup, table, tbody, td,
	template, textarea, tfoot, th, thead, time, title, tr, track, u, ul, var, video, wbr,
];
