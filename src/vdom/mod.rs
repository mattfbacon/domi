use std::cmp::Ordering;
use std::collections::HashMap;

use wasm_bindgen::JsCast as _;
use web_sys::Node;

pub use self::builder::DomBuilder;
use crate::event::{Id, ID_DATA_KEY};

pub mod builder;

#[derive(Debug)]
pub struct VNodeElement {
	pub(crate) id: Id,
	pub(crate) tag: String,
	// compromise: the map is not allocated in the arena
	pub(crate) attributes: HashMap<String, String>,
	pub(crate) children: Vec<VNode>,
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

		for child in &self.children {
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

struct MustRegenerate;

fn patch_dom_(dom: &Node, old: &[VNode], new: &[VNode]) -> Result<(), MustRegenerate> {
	let dom_children = dom.child_nodes();

	for (i, (old, new)) in old.iter().zip(new.iter()).enumerate() {
		let i = u32::try_from(i).unwrap();
		match (old, new) {
			(VNode::Text(old), VNode::Text(new)) => {
				if old != new {
					dom_children
						.item(i)
						.ok_or(MustRegenerate)?
						.set_node_value(Some(new));
				}
			}
			(VNode::Element(old), VNode::Element(new)) if old.tag == new.tag => {
				let dom_child = dom_children.item(i).ok_or(MustRegenerate)?;
				let dom_child = dom_child.dyn_into::<web_sys::HtmlElement>().unwrap();

				if old.id != new.id {
					dom_child
						.dataset()
						.set(ID_DATA_KEY, &new.id.to_string())
						.unwrap();
				}

				for removed in old
					.attributes
					.keys()
					.filter(|&attr| !new.attributes.contains_key(attr))
				{
					dom_child.remove_attribute(removed).unwrap();
				}
				for (added_or_modified, value) in new
					.attributes
					.iter()
					.filter(|&(attr, value)| old.attributes.get(attr) != Some(value))
				{
					dom_child.set_attribute(added_or_modified, value).unwrap();
				}

				patch_dom_(&dom_child, &old.children, &new.children)?;
			}
			(_, new) => {
				let dom_child = dom_children.item(i).ok_or(MustRegenerate)?;
				// new comes before old here
				dom.replace_child(&new.to_dom(), &dom_child).unwrap();
			}
		}
	}

	match new.len().cmp(&old.len()) {
		Ordering::Greater => {
			for new in new.iter().skip(old.len()) {
				dom.append_child(&new.to_dom()).unwrap();
			}
		}
		Ordering::Less => {
			for _ in new.len()..old.len() {
				dom
					.remove_child(&dom.last_child().ok_or(MustRegenerate)?)
					.unwrap();
			}
		}
		Ordering::Equal => {}
	}

	Ok(())
}

pub fn patch_dom(dom: &Node, old: &[VNode], new: &[VNode]) {
	match patch_dom_(dom, old, new) {
		Ok(()) => (),
		Err(MustRegenerate) => {
			while let Some(dom_child) = dom.first_child() {
				dom.remove_child(&dom_child).unwrap();
			}
			for node in new {
				dom.append_child(&node.to_dom()).unwrap();
			}
		}
	}
}
