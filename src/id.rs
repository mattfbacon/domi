use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Id(pub u64);

impl Id {
	pub(crate) const DATA_KEY: &str = "__domi_id";
}

impl std::fmt::Display for Id {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(formatter)
	}
}

impl Id {
	pub(crate) fn new(source: impl Hash) -> Self {
		Self(fxhash::hash64(&source))
	}

	pub(crate) fn with(self, source: impl Hash) -> Self {
		Self(fxhash::hash64(&(self.0, source)))
	}
}

/// A hasher intended to be used with `HashMap`s where `Id` is the key.
/// Since IDs are already hashed, there's no reason to hash them again.
#[derive(Default)]
pub(crate) struct PassthroughHasher(u64);

impl Hasher for PassthroughHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &byte in bytes {
			self.0 <<= 8;
			self.0 |= u64::from(byte);
		}
	}

	// A fast case for `u64`, which is what `Id` is.
	#[inline]
	fn write_u64(&mut self, v: u64) {
		self.0 = v;
	}
}
