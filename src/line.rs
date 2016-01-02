use std::error::Error;
use std::io::{self, Read, BufRead, Write, Stdin, Stdout, BufReader};
use std::path::Path;
use std::str;

use notty_encoding::args::{InputSettings, BufferSettings};
use notty_encoding::cmds::{SetInputMode, HoldForInput};

use util;
use util::tty::TtyGuard;

pub struct LineBuffer<I: Read, O: Write> {
    stdin: BufReader<I>,
    stdout: O,
    prompt: Vec<u8>,
    tty: TtyGuard,
}

impl LineBuffer<Stdin, Stdout> {
    pub fn new(prompt: &str) -> io::Result<LineBuffer<Stdin, Stdout>> {
        LineBuffer::open(io::stdin(), io::stdout(), prompt, "/dev/tty")
    }
}

impl<I, O> LineBuffer<I, O> where I: Read, O: Write {
    pub fn open<T: AsRef<Path>>(stdin: I, mut stdout: O, prompt: &str, tty: T)
            -> io::Result<LineBuffer<I, O>> {
        let tty = try!(TtyGuard::new(tty));
        try!(util::write_esc(&mut stdout, &SetInputMode(InputSettings::LineBufferEcho(
            tty.echo, tty.buffer
        ))));
        try!(stdout.flush());
        Ok(LineBuffer {
            stdin: BufReader::new(stdin),
            stdout: stdout,
            prompt: ["\n", prompt, " "].concat().into_bytes(),
            tty: tty,
        })
    }

    pub fn loop_with<E, F>(&mut self, mut func: F) -> Result<(), Box<Error>>
    where E: Error, F: FnMut(&str) -> Result<(), Box<Error>> {
        let mut buffer = String::new();
        loop {
            try!(self.read_line(&mut buffer));
            try!(func(&buffer));
            buffer.clear();
        }
    }

    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        try!(self.stdout.write_all(&self.prompt));
        try!(util::write_esc(&mut self.stdout, &HoldForInput));
        try!(self.stdout.flush());

        read_line(&mut self.stdin, unsafe { buf.as_mut_vec() }, self.tty.buffer).and_then(|n| {
            if str::from_utf8(buf.as_bytes()).is_ok() {
                Ok(n)
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData,
                                   "stream contained invalid utf8 data"))
            }
        })
    }

}

fn read_line<R: BufRead>(read: &mut R, buf: &mut Vec<u8>, set: BufferSettings)
-> io::Result<usize> {
    struct Guard<'a> { string: &'a mut Vec<u8>, init_len: usize }
    impl<'a> Drop for Guard<'a> {
        fn drop(&mut self) { unsafe { self.string.set_len(self.init_len); } }
    }
    let len = buf.len();
    let mut buffer = Guard { string: buf, init_len: len };
    loop {
        let (done, used) = {
            let available = match read.fill_buf() {
                Ok(buf) => buf,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e)  => return Err(e),
            };
            match available.iter().position(|&n| n == b'\n' || set.eol(n as char)) {
                Some(idx) if set.eof(available[idx] as char) => {
                    buffer.string.extend(&available[..idx]);
                    (true, idx + 1)
                }
                Some(idx)   => {
                    buffer.string.extend(&available[..idx+1]);
                    (true, idx + 1)
                }
                None        => {
                    buffer.string.extend(available);
                    (true, available.len())
                }
            }
        };
        read.consume(used);
        if done || used == 0 {
            let ret = buffer.string.len() - buffer.init_len;
            ::std::mem::forget(buffer);
            return Ok(ret)
        }
    }
}

impl<I, O> Write for LineBuffer<I, O> where I: Read, O: Write {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

impl<I, O> Drop for LineBuffer<I, O> where I: Read, O: Write {
    fn drop(&mut self) {
        let _ = util::write_esc(&mut self.stdout, &SetInputMode(InputSettings::Ansi(false)));
        let _ = self.stdout.flush();
    }
}
