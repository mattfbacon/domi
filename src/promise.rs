//! Provides the [`Promise`] abstraction to receive the result of an asynchronous operation.

use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use crate::Context;

/// A promise representing an asynchronous operation.
///
/// The main advantage of using this abstraction is that it automatically requests an update when the operation completes, so that the UI can update in response to the result.
///
/// There are a couple nuances.
/// If the operation panics, the promise will never complete.
/// Return a `Result` instead.
pub struct Promise<T> {
	place: Rc<RefCell<Option<T>>>,
}

impl<T: 'static> Promise<T> {
	/// Spawn an asynchronous operation, returning a [`Promise`] that allows receiving the result of the operation.
	#[must_use]
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

	/// Try to obtain the result of the operation.
	///
	/// The function returns `Some` when the result has been received or `None` if it has not yet been received.
	///
	/// The result can only be obtained once. Once the result has been obtained this function will always return `None`.
	/// Logically, this function consumes the [`Promise`]. However, it does not actually consume `self` to make it easier to work with.
	#[must_use]
	pub fn try_take(&mut self) -> Option<T> {
		self.place.borrow_mut().take()
	}
}
