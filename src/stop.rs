
use crate::{
	ParseIterator,
	recorder::Recorder
};

#[derive(Debug)]
pub struct Stop<'a, T> {
	inner: &'a mut T
}

impl<'a, T> Stop<'a, T> {
	pub(super) fn new(inner: &'a mut T) -> Self {
		Self {inner}
	}
}

impl<'s, 'a, T> ParseIterator<'s> for Stop<'a, T>
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
		None // stopped so don't continue
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
	fn test_stop() {

		let mut parser = StrParser::new("123456789");
		let mut parser = parser.record();

		parser.consume_len(5).unwrap();
		assert_eq!("12345", parser.to_str());
		assert_eq!( "12345", parser.stop().consume_to_str() );

	}

}