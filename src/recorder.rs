
use crate::{
	ParseIterator,
	position::Position,
	pit::PointInTime
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recorder {
	pos: Position
}

impl Recorder {

	pub(super) fn new(pos: Position) -> Self {
		Self {pos}
	}

	pub fn pos(&self) -> Position {
		self.pos
	}

}


#[derive(Debug)]
pub struct RecordIter<'a, T> {
	inner: &'a mut T,
	recorder: Recorder
}

impl<'s, 'a, T> RecordIter<'a, T>
where T: ParseIterator<'s> {
	pub(super) fn new(inner: &'a mut T) -> Self {
		let pos = inner.pit().record_pos();
		Self {
			inner,
			recorder: Recorder::new(pos)
		}
	}
}


impl<'s, 'a, T> ParseIterator<'s> for RecordIter<'a, T>
where T: ParseIterator<'s> {

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
		self.inner.advance()
	}

	fn recorder(&self) -> Option<&Recorder> {
		Some(&self.recorder)
	}

	// fn to_str(&self) -> Self::ToStrResult {
	// 	self.inner.to_str()
	// }

	#[inline]
	unsafe fn is_valid_utf8() -> bool {
		T::is_valid_utf8()
	}

}



#[cfg(test)]
mod tests {

	use crate::*;

	#[test]
	fn record() {

		let s = b"my byte str";

		let mut parser = Parser::new( s );
		parser.consume_while_byte_fn( |&b| b != b' ' )
			.next().unwrap();// skip first word

		unsafe {
			let st = parser.record()
				.consume_to_str_unchecked();

			assert_eq!( "byte str", st );
		}

	}

	#[test]
	fn record_combi() {

		let s = b"my byte str";

		let mut parser = Parser::new( s );
		let mut parser = parser.split_on_byte( b' ' );

		unsafe {
			let st = parser.next().unwrap()
				.record()
				.consume_to_str_unchecked();

			assert_eq!( "my", st );

			let st = parser.next().unwrap()
				.record()
				.consume_to_str_unchecked();

			assert_eq!( "byte", st );
		}

	}

	#[test]
	fn nested_record() {

		let s = b"aaaabbb";

		let mut parser = Parser::new( s );
		
		let mut first_recorder = parser.record();

		unsafe {
			let res = first_recorder
				.while_byte_fn( |&b| b == b'a' )
				.record()
				.consume_to_str_unchecked();
			assert_eq!( "aaaa", res );
		}

		unsafe {
			assert_eq!( "aaaabbb", first_recorder.consume_to_str_unchecked() );
		}

	}

	#[test]
	fn check_that_it_is_inplace() {

		let mut parser = Parser::new( b"aaaabbb" );

		unsafe {
			let res = parser
				.record()
				.consume_while_byte_fn( |&b| b == b'a' )
				.to_str_unchecked();
			assert_eq!( "aaaa", res );
		}

	}

}