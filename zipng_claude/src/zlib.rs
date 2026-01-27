use {
    crate::{
        adler32, default, generic::panic, output_buffer, write_deflate, DeflateMode, OutputBuffer,
    },
    std::io::Read,
};

#[cfg(feature = "flate2")]
pub fn read_zlib(input: &mut impl Read) -> Result<Vec<u8>, panic> {
    let mut buffer = Vec::new();
    flate2::read::ZlibDecoder::new(input).read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_zlib(output: &mut OutputBuffer, data: &[u8]) -> Result<usize, panic> {
    write_zlib {
        output,
        data,
        mode: default(),
    }
    .call()
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct write_zlib<'all> {
    pub output: &'all mut OutputBuffer,
    pub data: &'all [u8],
    pub mode: ZlibMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZlibMode {
    /// deflate with a 32KiB LZ77 window
    Deflate32K { mode: DeflateMode },
}

impl Default for ZlibMode {
    fn default() -> Self {
        Self::Deflate32K {
            mode: DeflateMode::default(),
        }
    }
}
impl write_zlib<'_> {
    pub fn call(&mut self) -> Result<usize, panic> {
        let Self {
            output,
            data,
            mode: _,
        } = self;

        let output = &mut *output.tagged("png", "zlib");

        let before = output.offset();

        // zlib compression mode: deflate with 32KiB windows
        let cmf = 0b_0111_1000;
        *output.tagged("png", "compression-mode") += &[cmf];
        // zlib flag bits: no preset dictionary, compression level 0
        let mut flg: u8 = 0b0000_0000;
        // zlib flag and check bits
        flg |= 0b1_1111 - ((((cmf as u16) << 8) | (flg as u16)) % 0b1_1111) as u8;
        *output.tagged("png", "bitflags") += &[flg];

        let mut buffer = output_buffer();
        write_deflate(&mut buffer, data)?;

        *output += &buffer;

        // adler-32 checksum of the deflated data
        *output.tagged("png", "checksum") += &adler32(buffer.as_ref()).to_le_bytes();

        Ok(output.offset() - before)
    }
}
