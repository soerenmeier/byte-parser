[![CI](https://github.com/soerenmeier/byte-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/soerenmeier/byte-parser/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/byte-parser)](https://crates.io/crates/byte-parser)
[![docs.rs](https://img.shields.io/docsrs/byte-parser)](https://docs.rs/byte-parser)

# Byte Parser
A library that provides a functional way to easely parse a string or a slice.

## Basic Example
```rust
use byte_parser::{StrParser, ParseIterator};

let mut parser = StrParser::new("\
    key: value\n\
    other key: more : value\n\
    also valid\
");

let lines: Vec<(&str, &str)> = parser
    .split_on_byte(b'\n')
    .map_and_collect(|line| {

        let key = line
            .record()
            .consume_while_byte_fn(|&b| b != b':')
            .to_str();

        let has_colon = line.advance().is_some();
        if !has_colon {
            return ("", key.trim_start());
        }

        let value = line
            .record()
            .consume_to_str();

        (key, value.trim_start())
    });

assert_eq!(lines[0], ("key", "value"));
assert_eq!(lines[1], ("other key", "more : value"));
assert_eq!(lines[2], ("", "also valid"));
```

## Example parsing a number
```rust
# use std::str::FromStr;
use byte_parser::{StrParser, ParseIterator};

#[derive(Debug, PartialEq)]
pub enum Number {
    Uint(usize),
    Integer(isize),
    Float(f32)
}

impl Number {
    /// # Panics
    /// Panics if invalid utf8 is found.
    /// Or if the digit is to large.
    pub fn from_parser<'s, I>(iter: &mut I) -> Option<Self>
    where I: ParseIterator<'s> {
        let mut iter = iter.record();

        // there could be a leading minus -
        let is_negative = iter
            .next_if(|&b| b == b'-')
            .is_some();

            // consume first digits
        iter
            .while_byte_fn(u8::is_ascii_digit)
            .consume_at_least(1)
                .ok()?;
            
        // there could be a dot
        let has_dot = iter
            .next_if(|&b| b == b'.')
            .is_some();

        if !has_dot {
            let s = iter.to_str();
            let num = match is_negative {
                true => Self::Integer(
                    s.parse().expect("digit to large")
                ),
                false => Self::Uint(
                    s.parse().expect("digit to large")
                )
            };

            return Some(num)
            }

            // consume next digits
        iter.consume_while_byte_fn(u8::is_ascii_digit);

        Some(Self::Float(
            iter.to_str().parse().expect("digit to large")
        ))
    }
}

impl FromStr for Number {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        let mut parser = StrParser::new(s);
            let num = Self::from_parser(&mut parser)
            .ok_or(())?;

        // parser not exhausted
        if parser.advance().is_some() {
            return Err(())
        }

        Ok(num)
    }
}

assert_eq!(Number::Float(1.23), "1.23".parse().unwrap());
assert_eq!(Number::Float(-32.1), "-32.1".parse().unwrap());
assert_eq!(Number::Uint(42), "42".parse().unwrap());
assert_eq!(Number::Integer(-42), "-42".parse().unwrap());
assert!(".42".parse::<Number>().is_err());
assert!("5.42 ".parse::<Number>().is_err());
```