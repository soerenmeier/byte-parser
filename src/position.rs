
use std::ops::{ Deref, Add };


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(Option<usize>);

impl Position {
	pub fn null() -> Self {
		Self( None )
	}

	pub fn opt(&self) -> Option<usize> {
		self.0
	}
}

impl Deref for Position {
	type Target = Option<usize>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Add<usize> for Position {
	type Output = usize;

	/// panics if you want to add zero on null
	fn add(self, other: usize) -> usize {
		match self.0 {
			Some(o) => o + other,
			None => other - 1
		}
	}
}

impl From<usize> for Position {
	fn from(n: usize) -> Self {
		Self(Some(n))
	}
}
