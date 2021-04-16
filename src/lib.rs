//! # Byte Parser
//! A library that provides a functional way to easely parse a string or a slice.
//!
//! ## Basic Example
//! ```
//! use byte_parser::{StrParser, ParseIterator};
//!
//! let mut parser = StrParser::new("\
//! 	key: value\n\
//! 	other key: more : value\n\
//! 	also valid\
//! ");
//!
//! let lines: Vec<(&str, &str)> = parser
//! 	.split_on_byte(b'\n')
//! 	.map_and_collect(|line| {
//!
//! 		let key = line
//! 			.record()
//! 			.consume_while_byte_fn(|&b| b != b':')
//! 			.to_str();
//!
//! 		let has_colon = line.advance().is_some();
//! 		if !has_colon {
//! 			return ("", key.trim_start());
//! 		}
//!
//! 		let value = line
//! 			.record()
//! 			.consume_to_str();
//!
//! 		(key, value.trim_start())
//! 	});
//!
//! assert_eq!(lines[0], ("key", "value"));
//! assert_eq!(lines[1], ("other key", "more : value"));
//! assert_eq!(lines[2], ("", "also valid"));
//! ```
//! 
//! ## Example parsing a number
//! ```
//! # use std::str::FromStr;
//! use byte_parser::{StrParser, ParseIterator};
//!
//! #[derive(Debug, PartialEq)]
//! pub enum Number {
//! 	Uint(usize),
//! 	Integer(isize),
//! 	Float(f32)
//! }
//!
//! impl Number {
//!		/// # Panics
//!		/// Panics if invalid utf8 is found.
//! 	/// Or if the digit is to large.
//! 	pub fn from_parser<'s, I>(iter: &mut I) -> Option<Self>
//! 	where I: ParseIterator<'s> {
//! 		let mut iter = iter.record();
//!
//! 		// there could be a leading minus -
//! 		let is_negative = iter
//! 			.next_if(|&b| b == b'-')
//! 			.is_some();
//!
//!			// consume first digits
//! 		iter
//! 			.while_byte_fn(u8::is_ascii_digit)
//! 			.consume_at_least(1)
//!				.ok()?;
//!			
//! 		// there could be a dot
//! 		let has_dot = iter
//! 			.next_if(|&b| b == b'.')
//! 			.is_some();
//!
//! 		if !has_dot {
//! 			let s = iter.to_str();
//! 			let num = match is_negative {
//! 				true => Self::Integer(
//! 					s.parse().expect("digit to large")
//! 				),
//! 				false => Self::Uint(
//! 					s.parse().expect("digit to large")
//! 				)
//! 			};
//!
//! 			return Some(num)
//!			}
//!
//!			// consume next digits
//! 		iter.consume_while_byte_fn(u8::is_ascii_digit);
//!
//! 		Some(Self::Float(
//! 			iter.to_str().parse().expect("digit to large")
//! 		))
//! 	}
//! }
//!
//! impl FromStr for Number {
//! 	type Err = ();
//! 	fn from_str(s: &str) -> Result<Self, ()> {
//! 		let mut parser = StrParser::new(s);
//!			let num = Self::from_parser(&mut parser)
//! 			.ok_or(())?;
//!
//! 		// parser not exhausted
//! 		if parser.advance().is_some() {
//! 			return Err(())
//! 		}
//!
//! 		Ok(num)
//! 	}
//! }
//!
//! assert_eq!(Number::Float(1.23), "1.23".parse().unwrap());
//! assert_eq!(Number::Float(-32.1), "-32.1".parse().unwrap());
//! assert_eq!(Number::Uint(420), "420".parse().unwrap());
//! assert_eq!(Number::Integer(-42), "-42".parse().unwrap());
//! assert!(".42".parse::<Number>().is_err());
//! assert!("5.42 ".parse::<Number>().is_err());
//! ```

pub mod position;
mod parse_iterator;
mod expect_byte;
pub mod ignore_byte;
pub mod while_byte_fn;
pub mod split_on_byte;
pub mod recorder;
pub mod stop;
pub mod pit;
#[cfg(feature = "unstable-parse-iter")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-parse-iter")))]
pub mod parse_iter;

pub use parse_iterator::ParseIterator;
pub use expect_byte::ExpectByte;
use recorder::Recorder;
use position::Position;
use pit::ParserPointInTime;

/// `ParseIterator` implementation for a slice.
#[derive(Debug)]
pub struct Parser<'s> {
	slice: &'s [u8],
	pit: ParserPointInTime
}

impl<'s> Parser<'s> {

	/// Creates a new `Parser` from a slice.
	pub fn new(slice: &'s [u8]) -> Self {
		Self {
			slice,
			pit: ParserPointInTime {
				pos: Position::null()
			}
		}
	}

}

impl<'s> ParseIterator<'s> for Parser<'s> {

	type PointInTime = ParserPointInTime;

	fn slice(&self) -> &'s [u8] {
		self.slice
	}

	fn pit(&self) -> Self::PointInTime {
		self.pit
	}

	fn restore_pit(&mut self, pit: Self::PointInTime) {
		self.pit = pit;
	}

	fn advance(&mut self) -> Option<()> {
		let n = self.pit.pos + 1;

		if n < self.slice.len() {
			self.pit.pos = n.into();
			Some(())
		} else {
			None
		}
	}

	fn recorder(&self) -> Option<&Recorder> {
		None
	}

	#[inline]
	unsafe fn is_valid_utf8() -> bool {
		false
	}

}


/// `ParseIterator` implementation for a str.
#[derive(Debug)]
pub struct StrParser<'s> {
	inner: &'s str,
	pit: ParserPointInTime
}

impl<'s> StrParser<'s> {

	/// Creates a new `StrParser` from a str.
	pub fn new(inner: &'s str) -> Self {
		Self {
			inner,
			pit: ParserPointInTime::new()
		}
	}

}

impl<'s> ParseIterator<'s> for StrParser<'s> {

	type PointInTime = ParserPointInTime;

	fn slice(&self) -> &'s [u8] {
		self.inner.as_bytes()
	}

	fn pit(&self) -> Self::PointInTime {
		self.pit
	}

	fn restore_pit(&mut self, pit: Self::PointInTime) {
		self.pit = pit;
	}

	fn advance(&mut self) -> Option<()> {
		let n = self.pit.pos + 1;

		if n < self.inner.len() {
			self.pit.pos = n.into();
			Some(())
		} else {
			None
		}
	}

	fn recorder(&self) -> Option<&Recorder> {
		None
	}

	#[inline]
	unsafe fn is_valid_utf8() -> bool {
		true
	}

}

#[cfg(feature = "unstable-parse-iter")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-parse-iter")))]
/// From a `ParseIterator` generate an `Iterator`.
/// 
/// ## Example parsing arguments
/// ```
/// # use byte_parser::{StrParser, ParseIterator, parse_iter};
/// fn args(arg_str: &str) -> impl Iterator<Item=&str> {
/// 	parse_iter(
/// 		StrParser::new(arg_str),
/// 		|parser| {
///
/// 			// if we are in a string
/// 			if parser.advance_if(|&b| b == b'\'')? {
/// 				let s = parser.record()
/// 					.while_byte_fn(|&b| b != b'\'')
/// 					.consume_to_str();
///
/// 				// consume the ending ' and
/// 				// if a space follows consume it to
/// 				parser.advance();
/// 				parser.advance_if(|&b| b == b' ');
/// 				return Some(s)
/// 			}
///
/// 			// consume until a whitespace or an '
/// 			let s = parser.record()
/// 				.while_byte_fn(|&b| !matches!(b, b' ' | b'\''))
/// 				.consume_to_str();
/// 			
/// 			// skip an empty space
/// 			parser.advance_if(|&b| b == b' ');
/// 
/// 			Some(s)
/// 		}
/// 	)
/// }
/// 
/// let args: Vec<_> = args("arg1 'arg 2' arg3'arg 4'arg5").collect();
/// assert_eq!(args, ["arg1", "arg 2", "arg3", "arg 4", "arg5"]);
/// ```
pub fn parse_iter<'s, I, F, O>(i: I, f: F) -> parse_iter::ParseIter<I, F>
where
	I: ParseIterator<'s>,
	F: FnMut(&mut I) -> Option<O> {
	parse_iter::ParseIter::new(i, f)
}

// TESTS
#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn parser_advance() {

		let s = b"my byte str";

		let mut parser = Parser::new(s);

		for b in s.iter() {
			assert_eq!(*b, parser.next().unwrap());
		}

		assert_eq!(None, parser.next());

	}

	#[test]
	fn str_parser_advance() {

		let s = "my byte str";

		let mut parser = StrParser::new(s);

		for b in s.as_bytes().iter() {
			assert_eq!(*b, parser.next().unwrap());
		}

		assert_eq!(None, parser.next());

	}

	#[test]
	fn str_parser_to_str() {

		let s = "my byte str";

		let mut parser = StrParser::new(s);
		parser.consume_len(3).unwrap();
		let mut parser = parser.record();
		assert_eq!("byte str", parser.consume_to_str());

	}

	#[test]
	fn create_an_iterator() {
		fn make_iter(s: &str) -> impl Iterator<Item=&str> {
			let mut parser = StrParser::new(s);
			std::iter::from_fn(move || {
				let mut split = parser.split_on_byte(b' ');
				let n = split.next()?;// probably split_on_byte to already move position to
				Some(n.record().consume_to_str())
			})
		}
		let mut iter = make_iter("my byte str");
		assert_eq!(iter.next().unwrap(), "my");
		assert_eq!(iter.next().unwrap(), "byte");
		assert_eq!(iter.next().unwrap(), "str");
		assert!(iter.next().is_none());
	}

	#[cfg(feature = "unstable-parse-iter")]
	#[test]
	fn test_parse_iter() {
		let mut iter = parse_iter(
			StrParser::new("my str"),
			|parser| {
				let mut split = parser.split_on_byte(b' ');
				split.next()
					.map(|n| n.record().consume_to_str())
			}
		);
		assert_eq!(iter.next().unwrap(), "my");
		assert_eq!(iter.next().unwrap(), "str");
		assert!(iter.next().is_none());
	}

}