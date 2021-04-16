
use crate::position::Position;

use std::fmt::Debug;

pub trait PointInTime: Debug + Copy + Eq {

	fn pos(&self) -> Position;

	unsafe fn set_pos(&mut self, pos: Position);

	#[inline]
	fn record_pos(&self) -> Position {
		self.pos()
	}

}

/// Default PointInTime Implementation. 
/// Used by Parser and StrParser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserPointInTime {
	pub(crate) pos: Position
}

impl ParserPointInTime {
	pub(crate) fn new() -> Self {
		Self {
			pos: Position::null()
		}
	}
}

impl PointInTime for ParserPointInTime {
	fn pos(&self) -> Position {
		self.pos
	}

	unsafe fn set_pos(&mut self, pos: Position) {
		self.pos = pos;
	}
}