use std::io;
use crate::internals::{State, Class, Mode};
use crate::internals::{STATE_TRANSITION_TABLE, ASCII_CLASS};

#[cfg(test)]
mod tests;
mod internals;

pub struct JsonChecker<R> {
    state: State,
    stack: Vec<Mode>,
    reader: R,
}

impl<R> JsonChecker<R> {
    /// new_JSON_checker starts the checking process by constructing a JSON_checker
    /// object. It takes a depth parameter that restricts the level of maximum
    /// nesting.
    ///
    /// To continue the process, call JSON_checker_char for each character in the
    /// JSON text, and then call JSON_checker_done to obtain the final result.
    /// These functions are fully reentrant.
    ///
    /// The JSON_checker object will be deleted by JSON_checker_done.
    /// JSON_checker_char will delete the JSON_checker object if it sees an error.
    pub fn new(reader: R) -> JsonChecker<R> {
        JsonChecker {
            state: State::Go,
            stack: vec![Mode::Done],
            reader,
        }
    }

    /// After calling new_JSON_checker, call this function for each character (or
    /// partial character) in your JSON text. It can accept UTF-8, UTF-16, or
    /// UTF-32. It returns TRUE if things are looking ok so far. If it rejects the
    /// text, it deletes the JSON_checker object and returns false.
    #[inline]
    fn next_byte(&mut self, next_byte: u8) -> bool {
        // Determine the character's class.
        let next_class = if next_byte >= 128 {
            Class::CEtc
        } else {
            ASCII_CLASS[next_byte as usize]
        };

        if next_class == Class::Invalid {
            return false;
        }

        // Get the next state from the state transition table.
        let next_state = STATE_TRANSITION_TABLE[self.state as usize][next_class as usize];
        if next_state.is_valid() {
            // Change the state.
            self.state = next_state;
        } else {
            // Or perform one of the actions.
            match next_state {
                State::Wec => { // Empty }
                    if !self.pop(Mode::Key) {
                        return false;
                    }
                    self.state = State::Ok;
                },
                State::Wcu => { // }
                    if !self.pop(Mode::Object) {
                        return false;
                    }
                    self.state = State::Ok;
                },
                State::Ws => { // ]
                    if !self.pop(Mode::Array) {
                        return false;
                    }
                    self.state = State::Ok;
                },
                State::Woc => { // {
                    if !self.push(Mode::Key) {
                        return false;
                    }
                    self.state = State::Ob;
                },
                State::Wos => { // [
                    if !self.push(Mode::Array) {
                        return false;
                    }
                    self.state = State::Ar;
                }
                State::Wq => { // "
                    match self.stack.last() {
                        Some(Mode::Key) => self.state = State::Co,
                        Some(Mode::Array) |
                        Some(Mode::Object) => self.state = State::Ok,
                        _ => return false,
                    }
                },
                State::Wcm => { // ,
                    match self.stack.last() {
                        Some(Mode::Object) => {
                            // A comma causes a flip from object mode to key mode.
                            if !self.pop(Mode::Object) || !self.push(Mode::Key) {
                                return false;
                            }
                            self.state = State::Ke;
                        }
                        Some(Mode::Array) => self.state = State::Va,
                        _ => return false,
                    }
                },
                State::Wcl => { // :
                    // A colon causes a flip from key mode to object mode.
                    if !self.pop(Mode::Key) || !self.push(Mode::Object) {
                        return false;
                    }
                    self.state = State::Va;
                },
                // Bad action.
                _ => return false,
            }
        }
        return true;
    }

    /// The JSON_checker_done function should be called after all of the characters
    /// have been processed, but only if every call to JSON_checker_char returned
    /// TRUE. This function deletes the JSON_checker and returns TRUE if the JSON
    /// text was accepted.
    pub fn finish(mut self) -> bool {
        self.state == State::Ok && self.pop(Mode::Done)
    }

    /// Push a mode onto the stack. Return false if there is overflow.
    fn push(&mut self, mode: Mode) -> bool {
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
            if !self.next_byte(*c) {
                return Err(io::Error::new(io::ErrorKind::Other, "invalid JSON data")); // TODO use a real error type
            }
        }

        Result::Ok(len)
    }
}
