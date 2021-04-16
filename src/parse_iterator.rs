
use crate::{
	pit::PointInTime,
	ignore_byte::IgnoreByte,
	while_byte_fn::WhileByteFn,
	split_on_byte::SplitOnByte,
	recorder::{Recorder, RecordIter},
	stop::Stop,
	expect_byte::ExpectByte
};

/// The main trait of this crate.
///
/// This trait allows to parse a slice or a str more easely.
///
/// This trait is lazy, if something should happen you need to `consume` it.  
/// For example, `to_str` only works if you call `record` and `consume` first.
pub trait ParseIterator<'s> {// s for slice

	/// The type that is used to store information about the current position.
	type PointInTime: PointInTime;

	/// Returns the full underlying slice.
	fn slice(&self) -> &'s [u8];

	/// Returns the current position. Should be used in combination with
	/// `restore_pit`.
	fn pit(&self) -> Self::PointInTime;

	/// Restore to a given position.
	///
	/// ## Warning
	/// Only call this method with a pit that was received from this instance.
	fn restore_pit(&mut self, pit: Self::PointInTime);

	/// Advances the internal position.
	fn advance(&mut self) -> Option<()>;

	/// Returns a `Recorder` if recording was started.
	fn recorder(&self) -> Option<&Recorder>;

	/// Advances if `advance_if` returns `true`. 
	/// Returns `None` if the iterator is empty.
	fn advance_if<F>(&mut self, advance_if: F) -> Option<bool>
	where F: FnOnce(&u8) -> bool {
		match advance_if(&self.peek()?) {
			true => self.advance().map(|_| true),
			false => Some(false)
		}
	}

	/// Returns the current byte if it exists.
	#[inline]
	fn byte(&self) -> Option<u8> {
		let pos = self.pit().pos().opt()?;
		self.slice().get(pos).map(|&a| a)
	}

	/// Returns the next byte if it exists and advances the internal position.
	#[inline]
	fn next(&mut self) -> Option<u8> {
		self.advance()?;
		// just check that everything works as expected
		debug_assert!(self.byte().is_some());
		self.byte()
	}

	/// Advances one if `next_if` returns `true`. 
	/// Returns `None` if did not advance.
	#[inline]
	fn next_if<F>(&mut self, next_if: F) -> Option<u8>
	where F: FnOnce(&u8) -> bool {
		match next_if(&self.peek()?) {
			true => self.next(),
			false => None
		}
	}

	/// Returns the next byte without advancing the internal position.
	#[inline]
	fn peek(&mut self) -> Option<u8> {
		let pit = self.pit();
		let n = self.next();
		self.restore_pit(pit);
		n
	}

	/// Returns the next x bytes without advancing the internal position.
	#[inline]
	fn peek_len(&mut self, len: usize) -> Option<&'s [u8]>
	where Self: Sized {
		let pit = self.pit();
		let s = self.record()
			.consume_len(len)
			.map(|iter| iter.to_slice())
			.ok();
		self.restore_pit(pit);
		s
	}

	/// Tries to get the byte at the given position, without advancing.
	#[inline]
	fn peek_at(&mut self, pos: usize) -> Option<u8> {
		assert!(pos > 0, "peek_at pos must be bigger than 0");

		let pit = self.pit();
		let n = self.consume_len(pos - 1).ok()
			.map(|p| p.next())
			.flatten();
		self.restore_pit(pit);
		n
	}

	/// Skips a given byte when calling next.
	///
	/// ## Warning
	/// If you later call `to_slice` or a similar methods
	/// the skipped byte will still be returned.
	///
	/// ## Example
	/// ```
	/// # use byte_parser::{Parser, ParseIterator};
	/// let mut parser = Parser::new(b"abc");
	/// let mut parser = parser.ignore_byte(b'b');
	/// assert_eq!(b'a', parser.next().unwrap());
	/// assert_eq!(b'c', parser.next().unwrap());
	/// ```
	#[inline]
	fn ignore_byte(&mut self, byte: u8) -> IgnoreByte<'_, Self>
	where Self: Sized {
		IgnoreByte::new(self, byte)
	}

	/// Advances while the function returns `true`.
	#[inline]
	fn while_byte_fn<F>(&mut self, f: F) -> WhileByteFn<'_, Self, F>
	where 
		Self: Sized,
		F: Fn(&u8) -> bool {
		WhileByteFn::new(self, f)
	}

	/// Consumes until the iterator is empty. 
	/// Meaning that `advance` returns None.
	#[inline]
	fn consume(&mut self) -> &mut Self {
		while let Some(_) = self.advance() {}
		self
	}

	/// Consumes until the iterator is empty,
	/// and returns how many times advance got called.
	#[inline]
	fn consume_and_count(&mut self) -> usize {
		let mut c = 0;
		while let Some(_) = self.advance() {
			c += 1
		}
		c
	}

	/// Consumes a given length. Returns how much was consumed if could
	/// not consume all.
	#[inline]
	fn consume_len(&mut self, len: usize) -> Result<&mut Self, usize> {
		// we can not just increase the internal position
		// because calling advance could increase the position more than once
		for i in 0..len {
			self.advance().ok_or(i)?;
		}

		Ok(self)
	}

	/// Consumes until the iterator is empty. 
	/// Returns `Err(len)` if could not consume `len`.
	#[inline]
	fn consume_at_least(&mut self, len: usize) -> Result<&mut Self, usize> {
		self.consume_len(len)?;
		Ok(self.consume())
	}

	/// Consumes until the iterator is empty, returning how much was consumed. 
	/// Returns `Err(len)` if could not consume `len`.
	#[inline]
	fn consume_at_least_and_count(&mut self, len: usize) -> Result<usize, usize> {
		self.consume_len(len)?;
		Ok(self.consume_and_count() + len)
	}

	/// Consumes while the function returns `true`.
	#[inline]
	fn consume_while_byte_fn<F>(&mut self, f: F) -> &mut Self
	where 
		Self: Sized,
		F: Fn(&u8) -> bool {
		self.while_byte_fn(f).consume();
		self
	}

	/// Consumes while a give `byte` is returned.
	#[inline]
	fn consume_while_byte(&mut self, byte: u8) -> &mut Self
	where Self: Sized {
		self.consume_while_byte_fn(|&b| b == byte)
	}

	// Consumes while an ascii whitespace is returned.
	// #[inline]
	// fn consume_while_ascii_whitespace(&mut self) -> &mut Self
	// where Self: Sized {
	// 	self.consume_while_byte_fn(u8::is_ascii_whitespace)
	// }

	/// Splits the iterator at a given byte.
	///
	/// ## Example
	/// ```
	/// # use byte_parser::{StrParser, ParseIterator};
	/// let mut parser = StrParser::new("Hello World!");
	/// let mut splitter = parser.split_on_byte(b' ');
	///
	/// let hello = splitter.next().unwrap()
	/// 	.record().consume_to_str();
	/// let world = splitter.next().unwrap()
	/// 	.record().consume_to_str();
	///
	/// assert_eq!(hello, "Hello");
	/// assert_eq!(world, "World!");
	/// assert!(splitter.next().is_none());
	/// ```
	#[inline]
	fn split_on_byte(&mut self, byte: u8) -> SplitOnByte<'_, Self>
	where Self: Sized {
		SplitOnByte::new(self, byte)
	}

	#[inline]
	fn count_byte(&mut self, byte: u8) -> usize
	where Self: Sized {
		self.while_byte_fn(|&b| b == byte)
			.consume_and_count()
	}

	/// Starts a new `Recorder` which starts recording at this position.
	#[inline]
	fn record(&mut self) -> RecordIter<'_, Self>
	where Self: Sized {
		RecordIter::new(self)
	}

	/// Returns a slice from the start of recording until now.
	///
	/// ## Panics
	/// If not called in context of a recorder. Meaning before
	/// calling `record`.
	#[inline]
	fn to_slice(&self) -> &'s [u8] {
		let start = self.recorder().expect("no recorder found").pos() + 1;
		let end = self.pit().record_pos() + 1;

		&self.slice()[start..end]
	}

	/// Returns a `str` from the start of recording until the current position
	/// without checking if the data is valid utf8.
	/// ## Panics
	/// Panics if not called after `record` gets called.
	/// ## Safety
	/// This function is safe if `Self::is_valid_utf8` returns `true`.
	#[inline]
	unsafe fn to_str_unchecked(&self) -> &'s str {
		std::str::from_utf8_unchecked(self.to_slice())
	}

	/// ## Safety
	/// Returning `false` is always safe. 
	/// If you return `true` the entire underlying slice must be valid utf8.
	unsafe fn is_valid_utf8() -> bool;

	/// Returns a `str` from the start of recording until the current position.
	///
	/// ## Example
	/// ```
	/// # use byte_parser::{Parser, StrParser, ParseIterator};
	/// let str_from_slice = Parser::new(b"abc")
	///		.record()
	/// 	.consume()
	/// 	.to_str();
	/// assert_eq!(str_from_slice, "abc");
	///
	/// let str_from_str = StrParser::new("abc")
	/// 	.record()
	/// 	.consume()
	/// 	.to_str();
	/// assert_eq!(str_from_str, "abc");
	/// ```
	///
	/// ## Panics
	/// Panics if not called after `record` was called. 
	/// Or if invalid utf8 is present.
	#[inline]
	fn to_str(&self) -> &'s str {
		if unsafe { Self::is_valid_utf8() } {
			// Safe because is_valid_utf8 guaranties everything is valid utf8
			unsafe { self.to_str_unchecked() }
		} else {
			std::str::from_utf8(self.to_slice()).expect("invalid utf8")
		}
	}

	/// Returns a `str` from the start of recording until the current position.
	///
	/// ## Example
	/// ```
	/// # use byte_parser::{Parser, StrParser, ParseIterator};
	/// let str_from_slice = Parser::new(b"abc")
	///		.record()
	/// 	.consume()
	/// 	.try_to_str().expect("slice is not valid utf8");
	/// assert_eq!(str_from_slice, "abc");
	///
	/// let str_from_str = StrParser::new("abc")
	/// 	.record()
	/// 	.consume()
	/// 		// can never return Err
	/// 	.try_to_str().unwrap();
	/// assert_eq!(str_from_str, "abc");
	/// ```
	///
	/// ## Panics
	/// Panics if not called after `record` was called.
	#[inline]
	fn try_to_str(&self) -> Result<&'s str, std::str::Utf8Error> {
		if unsafe { Self::is_valid_utf8() } {
			// Safe because is_valid_utf8 guaranties everything is valid utf8
			Ok(unsafe { self.to_str_unchecked() })
		} else {
			std::str::from_utf8(self.to_slice())
		}
	}

	/// Consumes the iterator and then returns a slice from the start of recording
	/// until the current position.
	///
	/// ## Panics
	/// Panics if not called after `record` was called.
	#[inline]
	fn consume_to_slice(&mut self) -> &'s [u8] {
		self.consume().to_slice()
	}

	/// Consumes the iterator and then returns a str from the start of recording
	/// until the current position. Without checking if the underlying data
	/// is valid utf8.
	///
	/// ## Panics
	/// Panics if not called after `record` was called.
	#[inline]
	unsafe fn consume_to_str_unchecked(&mut self) -> &'s str {
		self.consume().to_str_unchecked()
	}

	/// Consumes the iterator and then returns a str from the start of recording
	/// until the current position.
	///
	/// ## Panics
	/// Panics if not called after `record` was called or if the data contains invalid
	/// utf8.
	#[inline]
	fn consume_to_str(&mut self) -> &'s str {
		self.consume().to_str()
	}

	/// Consumes the iterator and then returns a str from the start of recording
	/// until the current position if the data is valid utf8.
	///
	/// ## Panics
	/// Panics if not called after `record` was called.
	#[inline]
	fn consume_try_to_str(&mut self) -> Result<&'s str, std::str::Utf8Error> {
		self.consume().try_to_str()
	}

	/// Returns ```&mut Self``` if the function returns `true` on the next byte.
	/// Else returns the byte that was received.
	#[inline]
	fn expect_byte_fn<F>(&mut self, f: F) -> Result<&mut Self, Option<u8>>
	where F: Fn(u8) -> bool {
		self.next()
			.expect_byte_fn(f)
			.map(|_| self)
	}

	/// Returns ```&mut Self``` if the function byte is equal to the next byte.
	/// Else returns the actual byte that was received.
	#[inline]
	fn expect_byte(&mut self, byte: u8) -> Result<&mut Self, Option<u8>> {
		self.expect_byte_fn(|b| b == byte)
	}

	/// Returns ```&mut Self``` if the end was reached (next returns None).
	#[inline]
	fn expect_none(&mut self) -> Result<&mut Self, u8> {
		match self.next() {
			Some(b) => Err(b),
			None => Ok(self)
		}
	}

	/// Returns a `ParseIterator` that always returns None.
	///
	/// ## Example
	/// ```
	/// # use byte_parser::{StrParser, ParseIterator};
	/// let mut s = StrParser::new("abc");
	/// assert_eq!(b'a', s.next().unwrap());
	/// let mut s = s.stop();
	/// assert!(s.next().is_none());
	/// ```
	#[inline]
	fn stop(&mut self) -> Stop<'_, Self>
	where Self: Sized {
		Stop::new(self)
	}

}

#[cfg(test)]
mod tests {

	use crate::*;

	#[test]
	fn test_count_byte() {

		let s = b"baaaab";

		let mut parser = Parser::new( s );
		assert_eq!( 0, parser.count_byte(b'a') );
		assert_eq!( b'b', parser.next().unwrap() );
		assert_eq!( 4, parser.count_byte(b'a') );
		assert_eq!( b'b', parser.next().unwrap() );
		assert!( parser.next().is_none() );
		assert_eq!( 0, parser.count_byte(b'a') );

	}

	#[test]
	fn combining_multiple_iters() {

		let s = b"ab\raaa\r aab\raa";

		Parser::new(s)
			.ignore_byte(b'\r')
			.split_on_byte(b' ')
			.for_each( |parser| {

				// lets ignore b
				// and count a
				let count_a = parser
					.ignore_byte(b'b')
					.count_byte(b'a');

				assert_eq!( count_a, 4 );

			} );

	}

	#[test]
	fn expect_byte() {

		let s = b"abaa";

		assert!( Parser::new(s)
			.expect_byte(b'a').unwrap()
			.expect_byte(b'a').is_err() );

	}

	#[test]
	fn advance_if() {

		let mut parser = Parser::new(b"ab");

		assert!(parser.advance_if(|&b| b == b'a').unwrap());
		assert!(!parser.advance_if(|&b| b == b'a').unwrap());
		assert!(parser.advance_if(|&b| b == b'b').unwrap());
		assert!(parser.advance_if(|&b| b == b'b').is_none());

	}

	#[test]
	fn next_if() {

		let mut parser = Parser::new(b"ab");

		assert_eq!(parser.next_if(|&b| b == b'a').unwrap(), b'a');
		assert!(parser.next_if(|&b| b == b'x').is_none());
		assert_eq!(parser.next_if(|&b| b == b'b').unwrap(), b'b');
		assert!(parser.next_if(|&b| b == b'x').is_none());

	}

	#[test]
	fn peek() {

		let s = b"abaa";

		assert_eq!( b'a', Parser::new(s).peek().unwrap() );
		assert_eq!( b'a', Parser::new(s).peek_at(1).unwrap() );
		assert_eq!( b'b', Parser::new(s).peek_at(2).unwrap() );
		assert_eq!( b'a', Parser::new(s).peek_at(3).unwrap() );
		assert!( Parser::new(s).peek_at(5).is_none() );

	}

	#[test]
	fn consume() {

		// normal
		let mut parser = Parser::new( b"aaa" );
		assert!( parser.consume().next().is_none() );

		// len
		let mut parser = Parser::new( b"aaa" );
		assert!( parser.consume_len( 1 ).unwrap().next().is_some() );

		let mut parser = Parser::new( b"aaa" );
		parser.consume();
		assert!(matches!( parser.consume_len(1), Err(0) ));

		// at least
		let mut parser = Parser::new( b"aaa" );
		assert!( parser.consume_at_least( 1 ).is_ok() );
		assert!( parser.next().is_none() );

	}

}