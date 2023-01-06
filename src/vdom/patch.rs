use std::cmp::Ordering;

use wasm_bindgen::JsCast as _;
use web_sys::Node;

use super::{VNode, VNodes};
use crate::event::ID_DATA_KEY;

/// Something is wrong in the DOM, probably due to tampering. It must be rebuilt entirely.
struct MustRegenerate;

fn patch_fallible(dom: &Node, old: &[VNode], new: &[VNode]) -> Result<(), MustRegenerate> {
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

				patch_fallible(&dom_child, &old.children.0, &new.children.0)?;
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

pub fn patch(dom: &Node, old: &VNodes, new: &VNodes) {
	match patch_fallible(dom, &old.0, &new.0) {
		Ok(()) => (),
		Err(MustRegenerate) => {
			while let Some(dom_child) = dom.first_child() {
				dom.remove_child(&dom_child).unwrap();
			}
			for node in &new.0 {
				dom.append_child(&node.to_dom()).unwrap();
			}
		}
	}
}
