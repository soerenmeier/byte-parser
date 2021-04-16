
use crate::ParseIterator;

#[cfg(feature = "unstable-parse-iter")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-parse-iter")))]
pub struct ParseIter<I, F> {
	i: I,
	f: F
}

#[cfg(feature = "unstable-parse-iter")]
impl<I, F> ParseIter<I, F> {
	pub(crate) fn new(i: I, f: F) -> Self {
		Self {i, f}
	}
}

#[cfg(feature = "unstable-parse-iter")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-parse-iter")))]
impl<'s, I, F, O> Iterator for ParseIter<I, F>
where
	I: ParseIterator<'s>,
	F: FnMut(&mut I) -> Option<O> {

	type Item = O;

	fn next(&mut self) -> Option<O> {
		(self.f)(&mut self.i)
	}
}