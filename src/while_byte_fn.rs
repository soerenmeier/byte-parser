
use crate::{
	ParseIterator,
	recorder::Recorder
};

#[derive(Debug)]
pub struct WhileByteFn<'a, T, F> {
	inner: &'a mut T,
	f: F
}

impl<'a, T, F> WhileByteFn<'a, T, F> {
	pub(super) fn new(inner: &'a mut T, f: F) -> Self {
		Self {inner, f}
	}
}

impl<'s, 'a, T, F> ParseIterator<'s> for WhileByteFn<'a, T, F>
where
	T: ParseIterator<'s>,
	F: Fn(&u8) -> bool {

	type PointInTime = T::PointInTime;

	fn slice(&self) -> &'s [u8] {
		self.inner.slice()
	}

	fn pit(&self) -> Self::PointInTime {
		self.inner.pit()
	}

	fn restore_pit(&mut self, pit: Self::PointInTime) {
		self.inner.restore_pit(pit)
	}

	fn advance(&mut self) -> Option<()> {
		let f = &self.f;
		let pit = self.inner.pit();
		let b = self.inner.next()?;

		if f(&b) {
			Some(())
		} else {
			self.inner.restore_pit(pit);
			None
		}
	}

	fn recorder(&self) -> Option<&Recorder> {
		self.inner.recorder()
	}

	#[inline]
	unsafe fn is_valid_utf8() -> bool {
		T::is_valid_utf8()
	}

}




#[cfg(test)]
mod tests {

	use crate::*;

	#[test]
	fn test_while_byte_fn() {

		let s = b"my str";

		let mut parser = Parser::new( s );
		let mut parser_while = parser.while_byte_fn( |&b| b != b' ' );

		assert_eq!( b'm', parser_while.next().unwrap() );
		assert_eq!( b'y', parser_while.next().unwrap() );
		assert!( parser_while.next().is_none() );

		let mut parser_while = parser.while_byte_fn( |&b| b != b' ' );
		assert!( parser_while.next().is_none() );// because we are at the space it should return none

		// skip space
		parser.next().unwrap();

		// now parse the rest
		let mut parser_while = parser.while_byte_fn( |&b| b != b' ' );

		assert_eq!( b's', parser_while.next().unwrap() );
		assert_eq!( b't', parser_while.next().unwrap() );
		assert_eq!( b'r', parser_while.next().unwrap() );
		assert!( parser_while.next().is_none() );

	}

}