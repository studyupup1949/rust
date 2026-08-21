/// Host Environment Interface
pub trait HostInterface {
    /// Print a string
    fn print(&mut self, string: &str, new_line: bool) -> std::io::Result<()>;

    /// Read a byte
    fn read_byte(&mut self) -> std::io::Result<u8>;

    /// This function should exit the program with specified code.
    ///
    /// For cases where exit is not desired, just let the function return
    /// and interpreter will terminate with an error.
    fn exit(&mut self, code: i32);
}

/// Standard [HostInterface] implementation
#[derive(Clone, Copy, Default)]
pub struct Standard;
impl HostInterface for Standard {
    fn print(&mut self, string: &str, new_line: bool) -> std::io::Result<()> {
        use std::io::Write;

        let mut stdout = std::io::stdout();
        stdout.write_all(string.as_bytes())?;
        if new_line {
            stdout.write_all(b"\n")?;
        }

        Ok(())
    }

    fn read_byte(&mut self) -> std::io::Result<u8> {
        use std::io::Read;

        let mut buf = [0];
        std::io::stdin().read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn exit(&mut self, code: i32) {
        std::process::exit(code);
    }
}
