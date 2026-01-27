use {
    crate::{generic::default, panic, OutputBuffer},
    std::{io::Read, ops::Not},
};

#[cfg(feature = "flate2")]
pub fn read_deflate(input: &mut impl Read) -> Result<Vec<u8>, panic> {
    let mut buffer = Vec::new();
    flate2::read::DeflateDecoder::new(input).read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_deflate(output: &mut OutputBuffer, data: &[u8]) -> Result<usize, panic> {
    write_deflate {
        output,
        data,
        mode: default(),
    }
    .call()
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct write_deflate<'all> {
    pub output: &'all mut OutputBuffer,
    pub data: &'all [u8],
    pub mode: DeflateMode,
}

impl<'all> write_deflate<'all> {
    pub fn call(&mut self) -> Result<usize, panic> {
        let Self {
            output,
            data,
            mode: DeflateMode::NoCompression { max_block_size },
        } = self;

        let before = output.offset();

        let chunks = data.chunks((*max_block_size).into());

        let count = chunks.len();
        for (index, chunk) in chunks.enumerate() {
            // deflate flag bits
            let is_last_chunk = index + 1 >= count;
            *output += &[is_last_chunk.into()];
            // deflate block length
            *output += &u16::try_from(chunk.len()).unwrap().to_le_bytes();
            // deflate block length check complement
            *output += &u16::try_from(chunk.len()).unwrap().not().to_le_bytes();

            *output += chunk;
        }

        Ok(output.offset() - before)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeflateMode {
    /// No compression. Only use "Non-compressed block (BTYPE=00)" blocks.
    NoCompression {
        /// The maximum number of bytes to include in each block.
        ///
        /// The maximum size of the block in the output stream will be this
        /// value plus five bytes for the block headers.
        max_block_size: u16,
    },
}

impl Default for DeflateMode {
    fn default() -> Self {
        Self::NoCompression {
            max_block_size: u16::MAX,
        }
    }
}
