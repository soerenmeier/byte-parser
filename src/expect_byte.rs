
type ExpResult<T> = Result<T, Option<u8>>;

pub trait ExpectByte: Sized {

	fn expect_byte_fn<F>(self, f: F) -> ExpResult<Self>
	where F: Fn(u8) -> bool;

	#[inline]
	fn expect_byte(self, byte: u8) -> ExpResult<Self> {
		self.expect_byte_fn(|b| b == byte)
	}

}

impl ExpectByte for Option<u8> {

	#[inline]
	fn expect_byte_fn<F>(self, f: F) -> ExpResult<Self>
	where F: Fn(u8) -> bool {
		match self {
			Some(b) if f(b) => Ok(self),
			_ => Err(self)
		}
	}

}

impl ExpectByte for u8 {

	#[inline]
	fn expect_byte_fn<F>(self, f: F) -> ExpResult<Self>
	where F: Fn(u8) -> bool {
		match f(self) {
			true => Ok(self),
			false => Err(Some(self))
		}
	}

}