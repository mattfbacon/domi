#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::hash::Hash;

use bumpalo::collections::Vec as BVec;

use crate::event::{Event, EventKind};
use crate::id::Id;
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

/// A helper type used to configure an element when building the DOM.
pub struct ElementBuilder<'a, 'x> {
	vdom: ElementOrId<'a, 'x>,
	shared: Shared<'a>,
}

/// Similar to `Cow<'_, str>`, but where the owned variant is `&'static str` instead of `String`.
#[derive(Debug, Clone, Copy)]
pub enum StaticOrDynamic<'a> {
	/// Static data.
	Static(&'static str),
	/// Borrowed data.
	Dynamic(&'a str),
}

impl StaticOrDynamic<'_> {
	fn copy_if_needed(self, bump: &bumpalo::Bump) -> &str {
		match self {
			Self::Static(pass_through) => pass_through,
			Self::Dynamic(must_copy) => bump.alloc_str(must_copy),
		}
	}
}

impl<'x> ElementBuilder<'_, 'x> {
	/// Remove an attribute from the element.
	///
	/// You probably shouldn't have to use this method.
	/// Prefer conditionally adding the attribute in the first place rather than conditionally removing it later.
	pub fn remove_attr(&mut self, attr: impl AsRef<str>) -> &mut Self {
		if let Some(vdom) = self.vdom.as_element() {
			vdom.attributes.remove(attr.as_ref());
		}
		self
	}

	/// Get a [`DomBuilder`] for the children of this element.
	///
	/// This method is the second part of the [`DomBuilder`]-[`ElementBuilder`] cycle.
	pub fn children(&mut self) -> DomBuilder<'_, 'x> {
		DomBuilder {
			parent_id: Some(self.vdom.id()),
			vdom: self.vdom.as_element().map(|element| &mut element.children),
			shared: self.shared,
		}
	}

	/// Check if the element was clicked.
	#[must_use]
	pub fn clicked(&self) -> bool {
		self
			.shared
			.event
			.filter(|event| event.target == self.vdom.id() && event.kind == EventKind::Click)
			.is_some()
	}
}

macro_rules! attr_methods {
	($($(#[$extra_meta:meta])* $vis:vis fn $method_name:ident(attr: $attr_kind:tt, value: $value_kind:tt);)*) => {
		impl<'x> ElementBuilder<'_, 'x> {
			$(
				/// Add an attribute to the element, replacing the old value if one was present.
				///
				/// This `attr_*` variant takes `attr`
				#[doc = attr_methods!(@describe $attr_kind)]
				/// and `value`
				#[doc = attr_methods!(@describe $value_kind)]
				/// .
				///
				$(#[$extra_meta])*
				$vis fn $method_name(&mut self, attr: attr_methods!(@ty $attr_kind), value: attr_methods!(@ty $value_kind)) -> &mut Self {
					if let Some(vdom) = self.vdom.as_element() {
						#[allow(unused_variables)]
						let bump = vdom.children.bump();
						vdom.attributes.insert(attr_methods!(@convert $attr_kind, attr, bump), attr_methods!(@convert $value_kind, value, bump));
					}
					self
				}
			)*
		}
	};

	(@ty static) => {
		&'static str
	};
	(@convert static, $v:expr, $bump:expr) => {
		$v
	};
	(@describe static) => {
		"statically"
	};

	(@ty anon) => {
		impl AsRef<str>
	};
	(@convert anon, $v:expr, $bump:expr) => {
		$bump.alloc_str($v.as_ref())
	};
	(@describe anon) => {
		"by making a copy"
	};

	(@ty explicit) => {
		StaticOrDynamic<'_>
	};
	(@convert explicit, $v:expr, $bump:expr) => {
		$v.copy_if_needed(&$bump)
	};
	(@describe explicit) => {
		"according to the [`StaticOrDynamic`] variant provided"
	};
}

attr_methods! {
	/// This is essentially the canonical `attr_*` variant, since it represents the most common use case: an attribute name known at compile-time with a value provided at runtime.
	pub fn attr(attr: static, value: anon);
	pub fn attr_static(attr: static, value: static);
	pub fn attr_dynamic(attr: anon, value: anon);
	/// This is the most flexible `attr_*` variant but also is the ugliest to call.
	pub fn attr_explicit(attr: explicit, value: explicit);
}

/// The main type used to build the DOM in the `run` callback.
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

	/// Add a text node with the provided `content`.
	pub fn text(&mut self, content: impl AsRef<str>) {
		if let Some(vdom) = &mut self.vdom {
			let bump = vdom.bump();
			vdom.push(VNode::Text(bump.alloc_str(content.as_ref())));
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

	/// Add a child element of the provided `tag` and with the provided `id`.
	///
	/// `id` is used internally and must be unique *within the direct children of the current element*.
	/// That is, they do not need to be globally unique, only locally unique.
	/// See the documentation in the crate root for more information about IDs.
	#[inline]
	pub fn element(&mut self, id: impl Hash, tag: impl AsRef<str>) -> ElementBuilder<'_, 'x> {
		self.element_(
			self
				.parent_id
				.map_or_else(|| Id::new(&id), |parent_id| parent_id.with(&id)),
			tag.as_ref(),
		)
	}

	/// Get a reference to the containing [`Context`].
	#[inline]
	#[must_use]
	pub fn context(&self) -> &'a Context {
		self.shared.context
	}
}
