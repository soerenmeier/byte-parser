//! 
//! Splits the iterator at a given byte.
//!
//! ## Example
//! ```
//! # use byte_parser::{StrParser, ParseIterator};
//! let mut parser = StrParser::new("Hello World!");
//! let mut splitter = parser.split_on_byte(b' ');
//!
//! let hello = splitter.next().unwrap()
//! 	.record().consume_to_str();
//! let world = splitter.next().unwrap()
//! 	.record().consume_to_str();
//!
//! assert_eq!(hello, "Hello");
//! assert_eq!(world, "World!");
//! assert!(splitter.next().is_none());
//! ```


use crate::{
	ParseIterator,
	recorder::Recorder,
	position::Position,
	pit::PointInTime
};

use std::iter;


#[derive(Debug)]
pub struct SplitOnByte<'a, T> {
	inner: SplitOnByteIter<'a, T>
}

impl<'s, 'a, T> SplitOnByte<'a, T>
where T: ParseIterator<'s> {
	pub(super) fn new(inner: &'a mut T, byte: u8) -> Self {
		Self {
			inner: SplitOnByteIter::new(inner, byte)
		}
	}
}

impl<'s, 'a, T> SplitOnByte<'a, T>
where T: ParseIterator<'s> {

	// next
	pub fn next(&mut self) -> Option<&mut SplitOnByteIter<'a, T>> {
		self.inner.reach_split_byte()?;
		self.inner.pit.record_pos = None;// can this break when we use revert?

		Some(&mut self.inner)
	}

	// for_each
	pub fn for_each<F>(&mut self, mut f: F) -> &mut Self
	where F: FnMut(&mut SplitOnByteIter<'a, T>) {

		let mut call_next = || {
			f(self.next()?);
			Some(())
		};

		// do while
		while let Some(_) = call_next() {}

		self
	}

	// map
	pub fn map_and_collect<F, A, B>(&mut self, mut f: F) -> B
	where
		F: FnMut(&mut SplitOnByteIter<'a, T>) -> A,
		B: iter::FromIterator<A> {
		iter::from_fn(|| {
			Some(f(self.next()?))
		})
		.collect()
	}

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitOnBytePointInTime {
	pos: Position,// this value should never be read unless it is returned from fn pit()
	byte_reached: bool,
	record_pos: Option<Position>// used so that we not return the split byte
}

impl PointInTime for SplitOnBytePointInTime {

	fn pos(&self) -> Position {
		self.pos
	}

	unsafe fn set_pos(&mut self, pos: Position) {
		self.pos = pos;
	}

	fn record_pos(&self) -> Position {
		match self.record_pos {
			Some(o) => o,
			None => self.pos
		}
	}

}


#[derive(Debug)]
pub struct SplitOnByteIter<'a, T> {
	inner: &'a mut T,
	byte: u8,
	pit: SplitOnBytePointInTime
}

impl<'s, 'a, T> SplitOnByteIter<'a, T>
where T: ParseIterator<'s> {
	pub(super) fn new(inner: &'a mut T, byte: u8) -> Self {

		let pit = SplitOnBytePointInTime {
			pos: inner.pit().pos(),
			byte_reached: true,// true so first call does not skip first 'iteration'
			record_pos: None
		};

		Self {inner, byte, pit}
	}

	pub(super) fn reach_split_byte(&mut self) -> Option<()> {

		// reach the byte if not already reached
		while let Some(_) = self.advance() {}

		if self.pit.byte_reached {// reset byte_reached
			self.pit.byte_reached = false;
			Some(())
		} else { // we reached the end
			None
		}
	}
}

impl<'s, 'a, T> ParseIterator<'s> for SplitOnByteIter<'a, T>
where T: ParseIterator<'s> {

	type PointInTime = SplitOnBytePointInTime;

	// returns the full slice not only the split slice
	fn slice(&self) -> &'s [u8] {
		self.inner.slice()
	}

	fn pit(&self) -> Self::PointInTime {
		self.pit
	}

	fn restore_pit(&mut self, pit: Self::PointInTime) {
		// the inner pit doesnt know that the position changed
		// safe because we just propagate our own position
		unsafe {
			let mut inner_pit = self.inner.pit();
			inner_pit.set_pos(pit.pos());
			self.inner.restore_pit(inner_pit);
		}
		self.pit = pit;
	}

	fn advance(&mut self) -> Option<()> {

		if self.pit.byte_reached {
			return None
		}

		let start = self.inner.pit().pos();
		self.inner.advance()?;

		self.pit.pos = self.inner.pit().pos();

		if self.byte().unwrap() == self.byte {
			self.pit.byte_reached = true;
			self.pit.record_pos = Some(start);
			None
		} else {
			self.pit.record_pos = None;
			Some(())
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
	fn test_split_on_byte_next() {

		let s = b"my byte str";

		let mut parser = Parser::new( s );
		let mut parser_split = parser.split_on_byte(b' ');

		let my = parser_split.next().unwrap();
		assert_eq!( b'm', my.next().unwrap() );
		assert_eq!( b'y', my.next().unwrap() );
		assert!( my.next().is_none() );

		let byte = parser_split.next().unwrap();
		assert_eq!( b'b', byte.next().unwrap() );
		assert_eq!( b'y', byte.next().unwrap() );
		// skip the rest
		//assert!( my.next().is_none() );

		let str_part = parser_split.next().unwrap();
		assert_eq!( b's', str_part.next().unwrap() );

		assert!( parser_split.next().is_none() );

	}

	#[test]
	fn test_split_on_byte_for_each() {

		let s = b"my byte str";

		let mut parser = Parser::new( s );
		let mut parser_while = parser.split_on_byte(b' ');

		let mut c = 0;
		parser_while.for_each( |_| {
			c += 1;
		} );

		assert_eq!( 3, c );

	}

	#[test]
	fn if_peek_called_could_mess_up_byte_reached() {
		// this test makes sure this doenst happen

		let s = b"ab\raaa\r aab\raa";

		Parser::new(s)
			.ignore_byte(b'\r')
			.split_on_byte(b' ')
			.for_each( |parser| {

				let a = parser
					.ignore_byte(b'b')
					.count_byte(b'a');
				assert_eq!( 4, a );

			} );

	}

}