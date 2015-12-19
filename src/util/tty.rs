use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use notty_encoding::args::{BufferSettings, EchoSettings};
use termios::{tcsetattr, Termios};
use termios::os::target::*;

pub struct TtyGuard {
    tty: PathBuf,
    pub buffer: BufferSettings,
    pub echo: EchoSettings
}

impl TtyGuard {

    pub fn new<T: AsRef<Path>>(tty: T) -> io::Result<TtyGuard> {
        let path = PathBuf::from(tty.as_ref());
        let tty = try!(File::open(&path));

        let mut termios = try!(Termios::from_fd(tty.as_raw_fd()));
        termios.c_lflag &= !(ICANON | ECHO);
        try!(tcsetattr(tty.as_raw_fd(), TCSAFLUSH, &termios));

        Ok(TtyGuard {
            tty: path,
            buffer: BufferSettings {
                eol1: termios.c_cc[VEOL],
                eol2: termios.c_cc[VEOL2],
                eof: termios.c_cc[VEOF],
                intr: termios.c_cc[VINTR],
                susp: termios.c_cc[VSUSP],
                quit: termios.c_cc[VQUIT],
            },
            echo: EchoSettings {
                lerase: termios.c_cc[VKILL],
                lnext: termios.c_cc[VLNEXT],
                werase: termios.c_cc[VWERASE],
            },
        })
    }

}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        let tty = match File::open(&self.tty) { Ok(tty) => tty, Err(_) => return };

        let mut termios = match Termios::from_fd(tty.as_raw_fd()) { Ok(t) => t, _ => return };
        termios.c_lflag |= ICANON | ECHO;
        let _ = tcsetattr(tty.as_raw_fd(), TCSAFLUSH, &termios);
    }
}
