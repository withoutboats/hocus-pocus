extern crate image as img;
extern crate notty_encoding;
extern crate termios;
extern crate termpix;

mod image;
mod line;
mod util;

pub use line::LineBuffer;
pub use image::print_image;

fn is_notty() -> bool {
    ::std::env::var("TERM").ok().map_or(false, |term| term.starts_with("notty"))
}
