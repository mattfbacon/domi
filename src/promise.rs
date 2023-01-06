use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use crate::Context;

pub struct Promise<T> {
	place: Rc<RefCell<Option<T>>>,
}

impl<T: 'static> Promise<T> {
	pub fn spawn_async<Fut>(fut: Fut, context: Context) -> Self
	where
		Fut: Future<Output = T> + 'static,
	{
		let place = Rc::new(RefCell::new(None));
		{
			let place = Rc::downgrade(&place);
			wasm_bindgen_futures::spawn_local(async move {
				let ret = fut.await;
				if let Some(place) = place.upgrade() {
					*place.borrow_mut() = Some(ret);
					context.request_update();
				}
			});
		}
		Self { place }
	}

	pub fn try_take(mut self) -> Result<T, Self> {
		self.try_take_by_ref().ok_or(self)
	}

	pub fn try_take_by_ref(&mut self) -> Option<T> {
		self.place.borrow_mut().take()
	}
}
