use std::io::{self, Write};

use notty_encoding::cmds::EscCode;

pub mod tty;

pub fn write_esc<W: Write, T: EscCode>(w: &mut W, cmd: &T) -> io::Result<()> {
    w.write_all(&cmd.encode().as_bytes())
}
