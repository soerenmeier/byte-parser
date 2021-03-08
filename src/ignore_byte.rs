
use crate::{
	ParseIterator,
	recorder::Recorder
};


#[derive(Debug)]
pub struct IgnoreByte<'a, T> {
	inner: &'a mut T,
	byte: u8
}

impl<'a, T> IgnoreByte<'a, T> {
	pub(super) fn new(inner: &'a mut T, byte: u8) -> Self {
		Self {inner, byte}
	}
}

impl<'s, 'a, T> ParseIterator<'s> for IgnoreByte<'a, T>
where T: ParseIterator<'s> {

	type PointInTime = T::PointInTime;

	fn slice(&self) -> &'s [u8] {
		self.inner.slice()
	}

	fn pit(&self) -> Self::PointInTime {
		self.inner.pit()
	}

	fn restore_pit(&mut self, pit: Self::PointInTime) {
		self.inner.restore_pit( pit )
	}

	fn advance(&mut self) -> Option<()> {
		let byte = self.byte;
		self.inner
			.while_byte_fn(|&b| b == byte)
			.consume();// consume
		self.inner.advance()
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
	fn test_ignore_byte() {

		let s = b"my byte str";

		let mut parser = Parser::new( s );
		let mut parser = parser.ignore_byte(b' ');

		for b in s.iter() {
			if *b == b' ' {
				continue
			}
			assert_eq!( *b, parser.next().unwrap() );
		}

		assert_eq!( None, parser.next() );

	}

}