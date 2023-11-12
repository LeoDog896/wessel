/// Emulates terminal. It holds input/output data in buffer
/// transferred to/from `Emulator`.
pub trait Terminal {
    /// Puts an output ascii byte data to output buffer.
    /// The data is expected to be read by user program via `get_output()`
    /// and be displayed to user.
    fn put_byte(&mut self, value: u8);
    /// Gets an input ascii byte data from input buffer.
    /// Used by `Emulator`.
    fn get_input(&mut self) -> u8;
}

/// For the test.
pub struct DummyTerminal {}

impl Default for DummyTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl DummyTerminal {
    pub fn new() -> Self {
        DummyTerminal {}
    }
}

impl Terminal for DummyTerminal {
    fn put_byte(&mut self, _value: u8) {}
    fn get_input(&mut self) -> u8 {
        0
    }
}
