use std::collections::HashMap;

use bumpalo::collections::Vec as BVec;
use bumpalo::Bump;
use wasm_bindgen::JsCast as _;
use web_sys::Node;

pub use self::builder::DomBuilder;
pub use self::patch::patch;
use crate::event::{Id, ID_DATA_KEY};

pub mod builder;
mod patch;

#[derive(Debug)]
pub struct VNodeElement<'x> {
	id: Id,
	tag: &'x str,
	attributes: HashMap<&'x str, &'x str>,
	children: BVec<'x, VNode<'x>>,
}

impl VNodeElement<'_> {
	fn to_dom(&self) -> Node {
		let element = web_sys::window()
			.unwrap()
			.document()
			.unwrap()
			.create_element(self.tag)
			.unwrap()
			.dyn_into::<web_sys::HtmlElement>()
			.unwrap();

		element
			.dataset()
			.set(ID_DATA_KEY, &self.id.to_string())
			.unwrap();

		for (attr, value) in &self.attributes {
			element.set_attribute(attr, value).unwrap();
		}

		for child in &self.children {
			element.append_child(&child.to_dom()).unwrap();
		}

		element.into()
	}
}

#[derive(Debug)]
pub enum VNode<'x> {
	Text(&'x str),
	Element(VNodeElement<'x>),
}

impl VNode<'_> {
	fn to_dom(&self) -> Node {
		match self {
			Self::Text(text) => web_sys::Text::new_with_data(text).unwrap().into(),
			Self::Element(element) => element.to_dom(),
		}
	}
}

/*
#[derive(Debug)]
#[ouroboros::self_referencing]
struct VNodesInner {
	arena: Bump,
	#[borrows(arena)]
	#[covariant]
	children: BVec<'this, VNode<'this>>,
}
*/

///Encapsulates implementation details for a self-referencing struct. This module is only visible when using --document-private-items.
mod ouroboros_impl_v_nodes_inner {
	#![allow(unsafe_code)]
	use super::{BVec, Bump, VNode};
	///The self-referencing struct.
	pub(super) struct VNodesInner {
		#[doc(hidden)]
		children: BVec<'static, VNode<'static>>,
		#[doc(hidden)]
		arena: ::ouroboros::macro_help::AliasableBox<Bump>,
	}
	fn _check_if_okay_according_to_checkers(
		arena: Bump,
		children_builder: impl for<'this> ::core::ops::FnOnce(&'this Bump) -> BVec<'this, VNode<'this>>,
	) {
		///A struct for holding immutable references to all [tail and immutably borrowed fields](https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html#definitions) in an instance of [`VNodesInner`](VNodesInner).
		pub(super) struct BorrowedFields<'outer_borrow, 'this>
		where
			'static: 'this,
		{
			pub(super) children: &'outer_borrow BVec<'this, VNode<'this>>,
			pub(super) arena: &'this Bump,
		}
		let arena = arena;
		let children = children_builder(&arena);
		let children = children;
		_ = BorrowedFields::<'_, '_> {
			arena: &arena,
			children: &children,
		};
	}
	///A more verbose but stable way to construct self-referencing structs. It is comparable to using `StructName { field1: value1, field2: value2 }` rather than `StructName::new(value1, value2)`. This has the dual benefit of making your code both easier to refactor and more readable. Call [`build()`](Self::build) to construct the actual struct. The fields of this struct should be used as follows:
	///
	///| Field | Suggested Use |
	///| --- | --- |
	///| `arena` | Directly pass in the value this field should contain |
	///| `children_builder` | Use a function or closure: `(arena: &_) -> children: _` |
	pub(super) struct VNodesInnerBuilder<
		ChildrenBuilder_: for<'this> ::core::ops::FnOnce(&'this Bump) -> BVec<'this, VNode<'this>>,
	> {
		pub(super) arena: Bump,
		pub(super) children_builder: ChildrenBuilder_,
	}
	impl<
			ChildrenBuilder_: for<'this> ::core::ops::FnOnce(&'this Bump) -> BVec<'this, VNode<'this>>,
		> VNodesInnerBuilder<ChildrenBuilder_>
	{
		///Calls [`VNodesInner::new()`](VNodesInner::new) using the provided values. This is preferrable over calling `new()` directly for the reasons listed above.
		pub(super) fn build(self) -> VNodesInner {
			VNodesInner::new(self.arena, self.children_builder)
		}
	}
	///A struct which contains only the [head fields](https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html#definitions) of [`VNodesInner`](VNodesInner).
	pub(super) struct Heads {
		pub(super) arena: Bump,
	}
	impl VNodesInner {
		pub(super) fn new(
			arena: Bump,
			children_builder: impl for<'this> ::core::ops::FnOnce(&'this Bump) -> BVec<'this, VNode<'this>>,
		) -> VNodesInner {
			let arena = ::ouroboros::macro_help::aliasable_boxed(arena);
			let arena_illegal_static_reference =
				unsafe { ::ouroboros::macro_help::change_lifetime(&*arena) };
			let children = children_builder(arena_illegal_static_reference);
			Self { children, arena }
		}
		///Provides an immutable reference to `children`. This method was generated because `children` is a [tail field](https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html#definitions).
		pub(super) fn borrow_children(&self) -> &BVec<'_, VNode<'_>> {
			&self.children
		}
		///Provides a mutable reference to `children`. This method was generated because `children` is a [tail field](https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html#definitions). No `borrow_children_mut` function was generated because Rust's borrow checker is currently unable to guarantee that such a method would be used safely.
		pub(super) fn with_children_mut<'outer_borrow, ReturnType>(
			&'outer_borrow mut self,
			user: impl for<'this> ::core::ops::FnOnce(
				&'outer_borrow mut BVec<'this, VNode<'this>>,
			) -> ReturnType,
		) -> ReturnType {
			user(&mut self.children)
		}
		///This function drops all internally referencing fields and returns only the [head fields](https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html#definitions) of this struct.
		#[allow(clippy::drop_ref)]
		#[allow(clippy::drop_copy)]
		#[allow(clippy::drop_non_drop)]
		pub(super) fn into_heads(self) -> Heads {
			::core::mem::drop(self.children);
			let arena = self.arena;
			Heads {
				arena: ::ouroboros::macro_help::unbox(arena),
			}
		}
	}
}
use ouroboros_impl_v_nodes_inner::{VNodesInner, VNodesInnerBuilder};

pub struct VNodes(VNodesInner);

impl Default for VNodes {
	fn default() -> Self {
		Self::with_arena(Bump::new())
	}
}

impl VNodes {
	pub fn with_arena(arena: Bump) -> Self {
		let builder = VNodesInnerBuilder {
			arena,
			children_builder: |arena| BVec::new_in(arena),
		};
		Self(builder.build())
	}

	fn clear_(self) -> Self {
		let heads = self.0.into_heads();
		let mut arena = heads.arena;
		arena.reset();
		Self::with_arena(arena)
	}

	pub fn clear(&mut self) {
		*self = std::mem::take(self).clear_();
	}

	pub fn children(&self) -> &[VNode<'_>] {
		self.0.borrow_children()
	}

	pub fn with_children_mut<'this, R>(
		&'this mut self,
		f: impl for<'x> FnOnce(&'this mut BVec<'x, VNode<'x>>) -> R,
	) -> R {
		self.0.with_children_mut(f)
	}
}
