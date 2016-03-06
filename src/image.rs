use std::io::{self, Write};

use img;
use termpix;

use notty_encoding::MediaFormat;
use notty_encoding::cmds::{PutMedia, EscCode};
use notty_encoding::args::MediaPosition;

pub fn print_image(data: Vec<u8>, width: u32, height: u32) {
    if super::is_notty() {
        let code = PutMedia::new(width, height,
                                 MediaPosition::Stretch, get_format(&data),
                                 data).encode();
        io::stdout().write(code.as_bytes()).unwrap_or_else(|e| panic!("{}", e));
    } else {
        let img = img::load_from_memory(&data).unwrap_or_else(|e| panic!("{}", e));
        termpix::print_image(img, false, width, height);
    }
}

fn get_format(data: &[u8]) -> MediaFormat {
    if &data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a" { MediaFormat::Gif }
    else if &data[0..8] == b"\x89PNG\r\n\x1a\n" { MediaFormat::Png }
    else if &data[0..3] == b"\xff\xd8\xff" { MediaFormat::Jpeg }
    else { panic!("Image format not supported.") }
}
