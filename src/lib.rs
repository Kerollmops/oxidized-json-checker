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

/// Represents any valid JSON type.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum JsonType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

/// A convenient method to check and consume JSON from a stream of bytes.
///
/// # Example
///
/// ```
/// # fn fmain() -> Result<(), Box<dyn std::error::Error>> {
/// use oxidized_json_checker::{validate, JsonType};
/// let text = r#""I am a simple string!""#;
/// let bytes = text.as_bytes();
///
/// let json_type = validate(bytes)?;
/// assert_eq!(json_type, JsonType::String);
/// # Ok(()) }
/// # fmain().unwrap()
/// ```
pub fn validate<R: io::Read>(reader: R) -> io::Result<JsonType> {
    let mut checker = JsonChecker::new(reader);
    io::copy(&mut checker, &mut io::sink())?;
    let outer_type = checker.finish()?;
    Ok(outer_type)
}

/// A convenient method to check and consume JSON from an `str`.
pub fn validate_str(string: &str) -> Result<JsonType, Error> {
    validate_bytes(string.as_bytes())
}

/// A convenient method to check and consume JSON from a bytes slice.
pub fn validate_bytes(bytes: &[u8]) -> Result<JsonType, Error> {
    let mut checker = JsonChecker::new(());
    checker.next_bytes(bytes)?;
    checker.finish()
}

/// The `JsonChecker` is a `io::Read` adapter, it can be used like a pipe,
/// reading bytes, checkings those and output the same bytes.
///
/// If an error is encountered, a JSON syntax error or an `io::Error`
/// it is returned by the `io::Read::read` method.
///
/// # Safety
///
/// An error encountered while reading bytes will invalidate the checker.
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
    error: Option<Error>,
    outer_type: Option<JsonType>,
    max_depth: usize,
    stack: Vec<Mode>,
    reader: R,
}

impl<R> fmt::Debug for JsonChecker<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JsonChecker").finish()
    }
}

impl<R> JsonChecker<R> {
    /// Construct a `JsonChecker. To continue the process, write to the `JsonChecker`
    /// like a sink, and then call `JsonChecker::finish` to obtain the final result.
    pub fn new(reader: R) -> JsonChecker<R> {
        JsonChecker::with_max_depth(reader, usize::max_value())
    }

    /// Construct a `JsonChecker` and restrict the level of maximum nesting.
    ///
    /// For more information read the `JsonChecker::new` documentation.
    pub fn with_max_depth(reader: R, max_depth: usize) -> JsonChecker<R> {
        JsonChecker {
            state: State::Go,
            error: None,
            outer_type: None,
            max_depth,
            stack: vec![Mode::Done],
            reader,
        }
    }

    #[inline]
    fn next_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        use packed_simd::u8x8;

        // TODO use chunks_exact instead?
        // By using u8x8 instead of u8x16 we lost 2s on 16s but
        // we are less prone to find state change requirements.
        for chunk in bytes.chunks(u8x8::lanes()) {
            if chunk.len() == u8x8::lanes() && self.state == State::St {
                // Load the bytes into a SIMD type
                let bytes = u8x8::from_slice_unaligned(chunk);

                // According to the state STATE_TRANSITION_TABLE we are in the `St` state
                // and *none of those bytes* are in the `CWhite`, `CQuote` or `CBacks` ascci class
                // we can avoid processing them at all because they will not change the current state.

                let cquotes = u8x8::splat(b'"');
                let cbacks = u8x8::splat(b'\\');

                let cwhites1 = u8x8::splat(b'\t');
                let cwhites2 = u8x8::splat(b'\n');
                let cwhites3 = u8x8::splat(b'\r');

                // We first compare with quotes because this is the most
                // common character we can encounter in valid JSON strings
                // and this way we are able to skip other comparisons faster
                if bytes.eq(cquotes).any() ||
                   bytes.eq(cbacks).any() ||
                   bytes.eq(cwhites1).any() ||
                   bytes.eq(cwhites2).any() ||
                   bytes.eq(cwhites3).any()
                {
                    chunk.iter().try_for_each(|b| self.next_byte(*b))?;
                }

                // Now that we checked that these bytes will not change
                // the state we can continue to the next chunk and ignore them

            } else {
                chunk.iter().try_for_each(|b| self.next_byte(*b))?;
            }
        }

        Ok(())
    }

    #[inline]
    fn next_byte(&mut self, next_byte: u8) -> Result<(), Error> {
        if let Some(error) = self.error {
            return Err(error);
        }

        // We can potentially use try_blocks in the future.
        fn internal_next_byte<R>(jc: &mut JsonChecker<R>, next_byte: u8) -> Result<(), Error> {
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
            let next_state = STATE_TRANSITION_TABLE[jc.state as usize][next_class as usize];

            // Save the type we met if not already saved.
            if jc.outer_type.is_none() {
                match next_state {
                    State::N1 => jc.outer_type = Some(JsonType::Null),
                    State::T1 | State::F1 => jc.outer_type = Some(JsonType::Bool),
                    State::In => jc.outer_type = Some(JsonType::Number),
                    State::Wq => jc.outer_type = Some(JsonType::String),
                    State::Wos => jc.outer_type = Some(JsonType::Array),
                    State::Woc => jc.outer_type = Some(JsonType::Object),
                    _ => (),
                }
            }

            match next_state {
                State::Wec => { // Empty }
                    if !jc.pop(Mode::Key) {
                        return Err(Error::EmptyCurlyBraces);
                    }
                    jc.state = State::Ok;
                },
                State::Wcu => { // }
                    if !jc.pop(Mode::Object) {
                        return Err(Error::OrphanCurlyBrace);
                    }
                    jc.state = State::Ok;
                },
                State::Ws => { // ]
                    if !jc.pop(Mode::Array) {
                        return Err(Error::OrphanSquareBrace);
                    }
                    jc.state = State::Ok;
                },
                State::Woc => { // {
                    if !jc.push(Mode::Key) {
                        return Err(Error::MaxDepthReached);
                    }
                    jc.state = State::Ob;
                },
                State::Wos => { // [
                    if !jc.push(Mode::Array) {
                        return Err(Error::MaxDepthReached);
                    }
                    jc.state = State::Ar;
                }
                State::Wq => { // "
                    match jc.stack.last() {
                        Some(Mode::Done) => {
                            if !jc.push(Mode::String) {
                                return Err(Error::MaxDepthReached);
                            }
                            jc.state = State::St;
                        },
                        Some(Mode::String) => {
                            jc.pop(Mode::String);
                            jc.state = State::Ok;
                        },
                        Some(Mode::Key) => jc.state = State::Co,
                        Some(Mode::Array) |
                        Some(Mode::Object) => jc.state = State::Ok,
                        _ => return Err(Error::InvalidQuote),
                    }
                },
                State::Wcm => { // ,
                    match jc.stack.last() {
                        Some(Mode::Object) => {
                            // A comma causes a flip from object mode to key mode.
                            if !jc.pop(Mode::Object) || !jc.push(Mode::Key) {
                                return Err(Error::InvalidComma);
                            }
                            jc.state = State::Ke;
                        }
                        Some(Mode::Array) => jc.state = State::Va,
                        _ => return Err(Error::InvalidComma),
                    }
                },
                State::Wcl => { // :
                    // A colon causes a flip from key mode to object mode.
                    if !jc.pop(Mode::Key) || !jc.push(Mode::Object) {
                        return Err(Error::InvalidColon);
                    }
                    jc.state = State::Va;
                },
                State::Invalid => {
                    return Err(Error::InvalidState)
                },

                // Or change the state.
                state => jc.state = state,
            }

            Ok(())
        }

        // By catching returned errors when this `JsonChecker` is used we *fuse*
        // the checker and ensure the user don't use a checker in an invalid state.
        if let Err(error) = internal_next_byte(self, next_byte) {
            self.error = Some(error);
            return Err(error);
        }

        Ok(())
    }

    /// The `JsonChecker::finish` method must be called after all of the characters
    /// have been processed.
    ///
    /// This function consumes the `JsonChecker` and returns `Ok(JsonType)` if the
    /// JSON text was accepted and the JSON type guessed.
    pub fn finish(self) -> Result<JsonType, Error> {
        self.into_inner().map(|(_, t)| t)
    }

    /// The `JsonChecker::into_inner` does the same as the `JsonChecker::finish`
    /// method but returns the internal reader along with the JSON type guessed.
    pub fn into_inner(mut self) -> Result<(R, JsonType), Error> {
        let is_state_valid = match self.state {
            State::Ok | State::In | State::Fr | State::Fs | State::E3 => true,
            _ => false,
        };

        if is_state_valid && self.pop(Mode::Done) {
            let outer_type = self.outer_type.expect("BUG: the outer type must have been guessed");
            return Ok((self.reader, outer_type))
        }

        // We do not need to catch this error to *fuse* the checker because this method
        // consumes the checker, it cannot be reused after an error has been thrown.
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
    /// Return false if the stack is empty or if the modes mismatch.
    fn pop(&mut self, mode: Mode) -> bool {
        self.stack.pop() == Some(mode)
    }
}

impl<R: io::Read> io::Read for JsonChecker<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If an error have already been encountered we return it,
        // this *fuses* the JsonChecker.
        if let Some(error) = self.error {
            return Err(error.into());
        }

        let len = match self.reader.read(buf) {
            Err(error) => {
                // We do not store the io::Error in the JsonChecker Error
                // type instead we use the IncompleteElement error.
                self.error = Some(Error::IncompleteElement);
                return Err(error);
            },
            Ok(len) => len,
        };

        self.next_bytes(&buf[..len])?;

        Ok(len)
    }
}
