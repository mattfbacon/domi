#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::hash::Hash;

use bumpalo::collections::Vec as BVec;

use crate::event::{Event, EventKind, Id};
use crate::vdom::{VNode, VNodeElement};
use crate::Context;

#[derive(Clone, Copy)]
struct Shared<'a> {
	event: Option<&'a Event>,
	context: &'a Context,
}

enum ElementOrId<'a, 'x> {
	Element(&'a mut VNodeElement<'x>),
	Id(Id),
}

impl<'x> ElementOrId<'_, 'x> {
	fn as_element(&mut self) -> Option<&mut VNodeElement<'x>> {
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

pub struct ElementBuilder<'a, 'x> {
	vdom: ElementOrId<'a, 'x>,
	shared: Shared<'a>,
}

impl<'x> ElementBuilder<'_, 'x> {
	pub fn attr(mut self, attr: impl AsRef<str>, value: impl AsRef<str>) -> Self {
		if let Some(vdom) = self.vdom.as_element() {
			let bump = vdom.children.bump();
			vdom.attributes.insert(
				bump.alloc_str(attr.as_ref()),
				bump.alloc_str(value.as_ref()),
			);
		}
		self
	}

	pub fn children(&mut self) -> DomBuilder<'_, 'x> {
		DomBuilder {
			parent_id: Some(self.vdom.id()),
			vdom: self.vdom.as_element().map(|element| &mut element.children),
			shared: self.shared,
		}
	}

	pub fn clicked(&self) -> bool {
		self
			.shared
			.event
			.filter(|event| event.target == self.vdom.id() && event.kind == EventKind::Click)
			.is_some()
	}
}

pub struct DomBuilder<'a, 'x> {
	parent_id: Option<Id>,
	vdom: Option<&'a mut BVec<'x, VNode<'x>>>,
	shared: Shared<'a>,
}

impl<'a, 'x> DomBuilder<'a, 'x> {
	/// If `None` is provided for `vdom`, then don't actually build a DOM, but still process events.
	pub(crate) fn new(
		vdom: Option<&'a mut BVec<'x, VNode<'x>>>,
		event: Option<&'a Event>,
		context: &'a Context,
	) -> Self {
		Self {
			parent_id: None,
			vdom,
			shared: Shared { event, context },
		}
	}

	pub fn text(&mut self, text: impl AsRef<str>) {
		if let Some(vdom) = &mut self.vdom {
			let bump = vdom.bump();
			vdom.push(VNode::Text(bump.alloc_str(text.as_ref())));
		}
	}

	fn element_(&mut self, id: Id, tag: &str) -> ElementBuilder<'_, 'x> {
		let inner = if let Some(vdom) = &mut self.vdom {
			let idx = vdom.len();
			vdom.push(VNode::Element(VNodeElement {
				id,
				tag: vdom.bump().alloc_str(tag),
				attributes: HashMap::new(),
				children: BVec::new_in(vdom.bump()),
			}));
			let VNode::Element(element) = &mut vdom[idx] else { unreachable!() };
			ElementOrId::Element(element)
		} else {
			ElementOrId::Id(id)
		};
		ElementBuilder {
			vdom: inner,
			shared: self.shared,
		}
	}

	#[inline]
	pub fn element(&mut self, id: impl Hash, tag: impl AsRef<str>) -> ElementBuilder<'_, 'x> {
		self.element_(
			self
				.parent_id
				.map_or_else(|| Id::new(&id), |parent_id| parent_id.with(&id)),
			tag.as_ref(),
		)
	}

	#[inline]
	#[must_use]
	pub fn context(&self) -> Context {
		self.shared.context.clone()
	}
}
