use std::collections::HashMap;

use wasm_bindgen::JsCast as _;
use web_sys::Node;

pub use self::builder::DomBuilder;
pub use self::patch::patch;
use crate::event::{Id, ID_DATA_KEY};

pub mod builder;
mod patch;

#[derive(Debug)]
pub struct VNodeElement {
	id: Id,
	tag: String,
	attributes: HashMap<String, String>,
	children: VNodes,
}

impl VNodeElement {
	fn to_dom(&self) -> Node {
		let element = web_sys::window()
			.unwrap()
			.document()
			.unwrap()
			.create_element(&self.tag)
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

		for child in &self.children.0 {
			element.append_child(&child.to_dom()).unwrap();
		}

		element.into()
	}
}

#[derive(Debug)]
pub enum VNode {
	Text(String),
	Element(VNodeElement),
}

impl VNode {
	fn to_dom(&self) -> Node {
		match self {
			Self::Text(text) => web_sys::Text::new_with_data(text).unwrap().into(),
			Self::Element(element) => element.to_dom(),
		}
	}
}

#[derive(Debug)]
pub struct VNodes(Vec<VNode>);

impl VNodes {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn clear(&mut self) {
		self.0.clear();
	}
}
