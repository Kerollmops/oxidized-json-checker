//! `oxidized-json-checker` is a library that provides JSON validation without
//! keeping the stream of bytes in memory, it streams the bytes and validate it
//! on the fly using a pushdown automaton.
//!
//! The original library has been retrieved from [json.org](http://www.json.org/JSON_checker/)
//! and improved to accept every valid JSON element has a valid JSOn document.
//!
//! Therefore this library accepts a single string or single integer as a valid JSON document,
//! this way we follow the [`serde_json`](https://docs.rs/serde_json) rules.
//!
//! # Example: validate some bytes
//!
//! This example shows how you can give the library a simple slice
//! of bytes and validate that it is a valid JSON document.
//!
//! ```
//! # fn fmain() -> Result<(), Box<dyn std::error::Error>> {
//! let text = r#"["I", "am", "a", "valid", "JSON", "array"]"#;
//! let bytes = text.as_bytes();
//!
//! oxidized_json_checker::validate(bytes)?;
//! # Ok(()) }
//! # fmain().unwrap()
//! ```
//!
//! # Example: validate a stream of bytes
//!
//! This example shows that you can use any type that implements `io::Read`
//! to the `JsonChecker` and validate that it is valid JSON.
//!
//! ```
//! # const json_bytes: &[u8] = b"null";
//! # fn streaming_from_the_web() -> std::io::Result<&'static [u8]> {
//! #     Ok(json_bytes)
//! # }
//! # fn fmain() -> Result<(), Box<dyn std::error::Error>> {
//! let stream = streaming_from_the_web()?;
//!
//! oxidized_json_checker::validate(stream)?;
//! # Ok(()) }
//! # fmain().unwrap()
//! ```
//!
//! # Example: complex compositions
//!
//! This example show how you can use the `JsonChecker` type to check
//! a compressed stream of bytes.
//!
//! You can decompress the stream, check it using the `JsonChecker`, and compress it
//! again to pipe it elsewhere. All of that without much memory impact.
//!
//! ```no_run
//! # fn fmain() -> Result<(), Box<dyn std::error::Error>> {
//! use std::io;
//! use oxidized_json_checker::JsonChecker;
//!
//! let stdin = io::stdin();
//! let stdout = io::stdout();
//!
//! // Wrap the stdin reader in a Snappy reader
//! // then wrap it in a JsonChecker reader.
//! let rdr = snap::read::FrameDecoder::new(stdin.lock());
//! let mut rdr = JsonChecker::new(rdr);
//!
//! // Wrap the stdout writer in a Snappy writer.
//! let mut wtr = snap::write::FrameEncoder::new(stdout.lock());
//!
//! // The copy function will return any io error thrown by any of the reader,
//! // the JsonChecker throw errors when invalid JSON is encountered.
//! io::copy(&mut rdr, &mut wtr)?;
//!
//! // We must check that the final bytes were valid.
//! rdr.finish()?;
//! # Ok(()) }
//! # fmain().unwrap()
//! ```
//!

use std::{fmt, io};
use crate::internals::{State, Class, Mode};
use crate::internals::{STATE_TRANSITION_TABLE, ASCII_CLASS};

#[cfg(test)]
mod tests;
mod internals;

/// The error type returned by the `JsonChecker` type.
#[derive(Copy, Clone, Debug)]
pub enum Error {
    InvalidCharacter,
    EmptyCurlyBraces,
    OrphanCurlyBrace,
    OrphanSquareBrace,
    MaxDepthReached,
    InvalidQuote,
    InvalidComma,
    InvalidColon,
    InvalidState,
    IncompleteElement,
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter => f.write_str("invalid character"),
            Error::EmptyCurlyBraces => f.write_str("empty curly braces"),
            Error::OrphanCurlyBrace => f.write_str("orphan curly brace"),
            Error::OrphanSquareBrace => f.write_str("orphan square brace"),
            Error::MaxDepthReached => f.write_str("max depth reached"),
            Error::InvalidQuote => f.write_str("invalid quote"),
            Error::InvalidComma => f.write_str("invalid comma"),
            Error::InvalidColon => f.write_str("invalid colon"),
            Error::InvalidState => f.write_str("invalid state"),
            Error::IncompleteElement => f.write_str("incomplete element"),
        }
    }
}

/// A convenient method to check and consume JSON from a stream of bytes.
///
/// # Example
///
/// ```
/// # fn fmain() -> Result<(), Box<dyn std::error::Error>> {
/// let text = r#""I am a simple string!""#;
/// let bytes = text.as_bytes();
///
/// oxidized_json_checker::validate(bytes)?;
/// # Ok(()) }
/// # fmain().unwrap()
/// ```
pub fn validate<R: io::Read>(reader: R) -> io::Result<()> {
    let mut checker = JsonChecker::new(reader);
    io::copy(&mut checker, &mut io::sink())?;
    checker.finish()?;
    Ok(())
}

/// The `JsonChecker` is a `io::Read` adapter, it can be used like a pipe,
/// reading bytes, checkings those and output the same bytes.
///
/// If an error is encountered, a JSON syntax error or an `io::Error`
/// it is returned by the `io::Read::read` method.
///
/// # Safety
///
/// It is invalid to call `JsonChecker::read`, `JsonChecker::finish` or `JsonChecker::into_inner`
    /// after an error has been returned by any method of this type.
///
/// # Example: read from a slice
///
/// ```
/// # fn fmain() -> Result<(), Box<dyn std::error::Error>> {
/// use std::io;
/// use oxidized_json_checker::JsonChecker;
///
/// let text = r#"{"I am": "an object"}"#;
/// let bytes = text.as_bytes();
///
/// let mut checker = JsonChecker::new(bytes);
/// io::copy(&mut checker, &mut io::sink())?;
/// checker.finish()?;
/// # Ok(()) }
/// # fmain().unwrap()
/// ```
pub struct JsonChecker<R> {
    state: State,
    max_depth: usize,
    stack: Vec<Mode>,
    reader: R,
}

impl<R> JsonChecker<R> {
    /// Construct a `JsonChecker. To continue the process, write to the `JsonChecker`
    /// like a sink, and then call `JsonChecker::finish` to obtain the final result.
    ///
    /// # Safety
    ///
    /// It is invalid to call `JsonChecker::read`, `JsonChecker::finish` or `JsonChecker::into_inner`
    /// after an error has been returned by any method of this type.
    pub fn new(reader: R) -> JsonChecker<R> {
        JsonChecker::with_max_depth(reader, usize::max_value())
    }

    /// Construct a `JsonChecker` and restrict the level of maximum nesting.
    ///
    /// For more information read the `JsonChecker::new` documentation.
    pub fn with_max_depth(reader: R, max_depth: usize) -> JsonChecker<R> {
        JsonChecker {
            state: State::Go,
            max_depth,
            stack: vec![Mode::Done],
            reader,
        }
    }

    #[inline]
    fn next_byte(&mut self, next_byte: u8) -> Result<(), Error> {
        // Determine the character's class.
        let next_class = if next_byte >= 128 {
            Class::CEtc
        } else {
            ASCII_CLASS[next_byte as usize]
        };

        if next_class == Class::Invalid {
            return Err(Error::InvalidCharacter);
        }

        // Get the next state from the state transition table and
        // perform one of the actions.
        let next_state = STATE_TRANSITION_TABLE[self.state as usize][next_class as usize];

        match next_state {
            State::Wec => { // Empty }
                if !self.pop(Mode::Key) {
                    return Err(Error::EmptyCurlyBraces);
                }
                self.state = State::Ok;
            },
            State::Wcu => { // }
                if !self.pop(Mode::Object) {
                    return Err(Error::OrphanCurlyBrace);
                }
                self.state = State::Ok;
            },
            State::Ws => { // ]
                if !self.pop(Mode::Array) {
                    return Err(Error::OrphanSquareBrace);
                }
                self.state = State::Ok;
            },
            State::Woc => { // {
                if !self.push(Mode::Key) {
                    return Err(Error::MaxDepthReached);
                }
                self.state = State::Ob;
            },
            State::Wos => { // [
                if !self.push(Mode::Array) {
                    return Err(Error::MaxDepthReached);
                }
                self.state = State::Ar;
            }
            State::Wq => { // "
                match self.stack.last() {
                    Some(Mode::Done) => {
                        if !self.push(Mode::String) {
                            return Err(Error::MaxDepthReached);
                        }
                        self.state = State::St;
                    },
                    Some(Mode::String) => {
                        self.pop(Mode::String);
                        self.state = State::Ok;
                    },
                    Some(Mode::Key) => self.state = State::Co,
                    Some(Mode::Array) |
                    Some(Mode::Object) => self.state = State::Ok,
                    _ => return Err(Error::InvalidQuote),
                }
            },
            State::Wcm => { // ,
                match self.stack.last() {
                    Some(Mode::Object) => {
                        // A comma causes a flip from object mode to key mode.
                        if !self.pop(Mode::Object) || !self.push(Mode::Key) {
                            return Err(Error::InvalidComma);
                        }
                        self.state = State::Ke;
                    }
                    Some(Mode::Array) => self.state = State::Va,
                    _ => return Err(Error::InvalidComma),
                }
            },
            State::Wcl => { // :
                // A colon causes a flip from key mode to object mode.
                if !self.pop(Mode::Key) || !self.push(Mode::Object) {
                    return Err(Error::InvalidColon);
                }
                self.state = State::Va;
            },
            State::Invalid => {
                return Err(Error::InvalidState)
            },

            // Or change the state.
            state => self.state = state,
        }

        Ok(())
    }

    /// The `JsonChecker::finish` method must be called after all of the characters
    /// have been processed, but only if there where no error thrown already.
    ///
    /// This function consumes the `JsonChecker` and returns `Ok(())` if the JSON
    /// text was accepted.
    ///
    /// # Safety
    ///
    /// It is invalid to call `JsonChecker::read`, `JsonChecker::finish` or `JsonChecker::into_inner`
    /// after an error has been returned by any method of this type.
    pub fn finish(self) -> Result<(), Error> {
        self.into_inner().map(drop)
    }

    /// The `JsonChecker::into_inner` does the same as the `JsonChecker::finish`
    /// method but returns the internal reader.
    ///
    /// # Safety
    ///
    /// It is invalid to call `JsonChecker::read`, `JsonChecker::finish` or `JsonChecker::into_inner`
    /// after an error has been returned by any method of this type.
    pub fn into_inner(mut self) -> Result<R, Error> {
        let is_state_valid = match self.state {
            State::Ok | State::In | State::Fr | State::Fs | State::E3 => true,
            _ => false,
        };

        if is_state_valid && self.pop(Mode::Done) {
            return Ok(self.reader)
        }

        Err(Error::IncompleteElement)
    }

    /// Push a mode onto the stack. Returns false if max depth is reached.
    fn push(&mut self, mode: Mode) -> bool {
        if self.stack.len() + 1 >= self.max_depth {
            return false;
        }
        self.stack.push(mode);
        return true;
    }

    /// Pop the stack, assuring that the current mode matches the expectation.
    /// Return false if there is underflow or if the modes mismatch.
    fn pop(&mut self, mode: Mode) -> bool {
        self.stack.pop() == Some(mode)
    }
}

impl<R: io::Read> io::Read for JsonChecker<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;

        for c in &buf[..len] {
            self.next_byte(*c)?;
        }

        Ok(len)
    }
}
