//! In-memory representation of a PNG image.

use {
    crate::{
        never, output_buffer,
        palettes::{crameri::OLERON, oceanic::TOPO, singles::FOUR_BIT_RAINBOW},
        panic, OutputBuffer, ToPng,
    },
    bitvec::slice::BitSlice,
    serde::{Deserialize, Serialize},
    std::io::Read,
    tracing::{instrument, trace},
};

#[doc(hidden)]
pub use self::{BitDepth::*, ColorType::*};

/// In-memory representation of a PNG file's essential image contents.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[non_exhaustive]
pub struct Png {
    pub pixel_data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bit_depth: BitDepth,
    pub color_type: ColorType,
    pub palette_data: Option<Vec<u8>>,
    pub transparency_data: Option<Vec<u8>>,
}

impl Png {
    #[instrument(skip_all)]
    /// Creates a new [`Png`] from the given data.
    pub fn new(data: &impl ToPng) -> Self {
        data.to_png().into_owned()
    }

    /// Create a new indexed-color [`Png`] with the given dimensions, bit depth,
    /// and color palette.
    pub fn new_indexed(width: usize, height: usize, bit_depth: BitDepth, palette: &[u8]) -> Self {
        let palette_data = Some(palette.to_vec());
        let pixel_count = width * height;
        let pixel_data =
            vec![
                0;
                pixel_count * bit_depth.bits_per_sample() * ColorType::Indexed.samples_per_pixel()
                    / 8
            ];
        Png {
            width,
            height,
            bit_depth,
            color_type: ColorType::Indexed,
            palette_data,
            transparency_data: None,
            pixel_data,
        }
    }

    /// Create a new greyscale [`Png`] with the given dimensions and bit depth.
    pub fn new_grayscale(width: usize, height: usize, bit_depth: BitDepth) -> Self {
        let color_type = Luminance;
        let pixel_data = vec![0u8; width * height * color_type.samples_per_pixel()];
        Png {
            pixel_data,
            width,
            height,
            bit_depth,
            color_type,
            palette_data: None,
            transparency_data: None,
        }
    }

    /// Creates a new 24-bit RGB [`Png`] with the given dimensions.
    pub fn new_rgb(width: usize, height: usize) -> Self {
        let bit_depth = EightBit;
        let color_type = RedGreenBlue;
        let pixel_data = vec![0u8; width * height * color_type.samples_per_pixel()];
        Png {
            pixel_data,
            width,
            height,
            bit_depth,
            color_type,
            palette_data: None,
            transparency_data: None,
        }
    }

    /// Creates a new 32-bit RGBA [`Png`] with the given dimensions.
    pub fn new_rgba(width: usize, height: usize) -> Self {
        let bit_depth = EightBit;
        let color_type = RedGreenBlueAlpha;
        let pixel_data = vec![0u8; width * height * color_type.samples_per_pixel()];
        Png {
            pixel_data,
            width,
            height,
            bit_depth,
            color_type,
            palette_data: None,
            transparency_data: None,
        }
    }

    #[instrument(skip_all)]
    /// Create a [`Png`] from unstructured image data bytes.
    ///
    /// Data beyond the first 32MiB may be ignored.
    pub fn from_unstructured_bytes(bytes: &[u8]) -> Self {
        let bit_depth;
        let color_type;
        let palette_data;
        let transparency_data = None;
        match bytes.len() {
            _len @ 0x0..=0x20 => {
                palette_data = None;
                bit_depth = OneBit;
                color_type = Luminance;
            },
            0x21..=0x100 => {
                palette_data = None;
                bit_depth = TwoBit;
                color_type = Luminance;
            },
            0x101..=0x100000 => {
                palette_data = Some(FOUR_BIT_RAINBOW.to_vec());
                bit_depth = FourBit;
                color_type = Indexed;
            },
            0x100001..=0x400000 => {
                palette_data = Some(OLERON.to_vec());
                bit_depth = BitDepth::EightBit;
                color_type = ColorType::Indexed;
            },
            0x400001..=0x800000 => {
                palette_data = Some(TOPO.to_vec());
                bit_depth = BitDepth::EightBit;
                color_type = ColorType::Indexed;
            },
            _ => {
                palette_data = None;
                bit_depth = BitDepth::EightBit;
                color_type = RedGreenBlueAlpha;
            },
        }
        let pixel_count =
            bytes.len() * 8 / (bit_depth.bits_per_sample() * color_type.samples_per_pixel());

        let width = 2048.min(((pixel_count as f64 + 1.).sqrt() as usize).next_power_of_two());
        let height = 8192.min((pixel_count + width - 1) / width);

        trace!(
            width = width,
            height = height,
            bit_depth = format!("{bit_depth:?}"),
            color_type = format!("{color_type:?}"),
            image_data_len = bytes.len(),
            "Creating Png from unstructured bytes",
        );

        Png {
            width,
            height,
            bit_depth,
            color_type,
            palette_data,
            transparency_data,
            pixel_data: bytes.to_vec(),
        }
    }

    /// Returns the number of bits per pixel in the image data of this [`Png`].
    ///
    /// This does not relate to the color palette (if one is present).
    pub fn bits_per_pixel(&self) -> usize {
        self.bit_depth.bits_per_sample() * self.color_type.samples_per_pixel()
    }

    /// Returns the number of bytes per row of pixels in the image data of this
    /// [`Png`].
    pub fn image_bytes_per_row(&self) -> usize {
        (self.width * self.bits_per_pixel() + 7) / 8
    }

    /// Returns the total number of image bytes that we expect the image to
    /// contain based on its metadata. **This may not be the actual size of
    /// the image data in this [`Png`]**, but in most cases it should be.
    pub fn image_bytes_expected(&self) -> usize {
        self.image_bytes_per_row() * self.height
    }

    #[instrument(skip_all)]
    /// Serializes this [`Png`] as a PNG image file.
    pub fn serialize(&self) -> OutputBuffer {
        let mut buffer = output_buffer();
        crate::png::write_png::write_png(
            &mut buffer,
            self.pixel_data.as_slice(),
            self.width.try_into().unwrap(),
            self.height.try_into().unwrap(),
            self.bit_depth,
            self.color_type,
            self.palette_data.as_deref(),
        )
        .unwrap();
        buffer
    }

    #[cfg(feature = "flate2")]
    #[instrument(skip_all)]
    /// Deserializes a PNG image file into a [`Png`].
    pub fn read(_input: &impl Read) -> Result<Self, panic> {
        unimplemented!()
    }

    #[cfg(feature = "flate2")]
    /// Deserialize a PNG image file into a [`Png`] from a byte vector.
    pub fn read_slice(input: &[u8]) -> Result<Self, never> {
        Ok(Self::read(&input)?)
    }

    pub fn mut_pixel(&mut self, x: usize, y: usize) -> &mut [u8] {
        if !matches!(self.bit_depth, EightBit | SixteenBit) {
            unimplemented!(
                "mut_pixel only supports 8- and 16-bit images, use mut_pixel_bits instead"
            );
        }
        let bits_per_pixel = self.bits_per_pixel();
        let row = y * self.image_bytes_per_row();
        let col = x * bits_per_pixel / 8;
        &mut self.pixel_data[row + col..row + col + bits_per_pixel / 8]
    }

    pub fn mut_pixel_bits(&mut self, _x: usize, _y: usize) -> &mut BitSlice<u8> {
        todo!()
    }

    /// Sets the pixel at the given coordinates to the given color.
    /// The required length of the color data may vary depending on the
    /// color type and bit depth of the image.
    ///
    /// This is provided for convenience, but it's not fast. If you need speed,
    /// modify the image data directly or use a faster library.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: &[u8]) -> Result<(), never> {
        self.mut_pixel(x, y).copy_from_slice(color);
        Ok(())
    }

    /// Sets the pixel at the given coordinates to the given color.
    /// The required length of the color data may vary depending on the
    /// color type and bit depth of the image.
    ///
    /// This is provided for convenience, but it's not fast. If you need speed,
    /// modify the image data directly or use a faster library.
    pub fn set_pixel_bits(
        &mut self,
        _x: usize,
        _y: usize,
        _color: &BitSlice<u8>,
    ) -> Result<(), never> {
        unimplemented!()
    }
}

impl Serialize for Png {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_bytes(self.serialize().as_ref())
    }
}

impl<'de> Deserialize<'de> for Png {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let bytes: &[u8] = serde_bytes::deserialize(deserializer)?;
        Self::read_slice(bytes).map_err(serde::de::Error::custom)
    }
}

/// The bit depth of an image, as defined in the PNG specification.
///
/// > **bit depth**: for indexed-colour images, the number of bits per palette
/// > index. For other images, the number of bits per sample in the image. This
/// > is the value that appears in the `IHDR` Image header chunk.
///
/// > **sample**: intersection of a channel and a pixel in an image.
///
/// > **channel**: array of all per-pixel information of a particular kind
/// > within a reference image. There are five kinds of information: red, green,
/// > blue, greyscale, and alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum BitDepth {
    OneBit = 1,
    TwoBit = 2,
    FourBit = 4,
    #[default]
    EightBit = 8,
    SixteenBit = 16,
}

/// The color type of an image, as defined in the PNG specification.
///
/// > There are five types of PNG image. Corresponding to each type is a
/// > **colour type**, which is the sum of the following values: 1 (palette
/// > used), 2 (truecolour used) and 4 (alpha used). Greyscale and truecolour
/// > images may have an explicit alpha channel. The PNG image types and
/// > corresponding colour types are listed in Table 8.
///
/// > **greyscale**: image representation in which each pixel is defined by a
/// > single sample of colour information, representing overall luminance (on a
/// > scale from black to white)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum ColorType {
    #[default]
    Luminance = 0,
    RedGreenBlue = 2,
    Indexed = 3,
    LuminanceAlpha = 4,
    RedGreenBlueAlpha = 6,
}

impl BitDepth {
    pub fn bits_per_sample(&self) -> usize {
        u8::from(*self).into()
    }
}

impl From<BitDepth> for u8 {
    fn from(depth: BitDepth) -> Self {
        depth as u8
    }
}

impl ColorType {
    pub fn samples_per_pixel(&self) -> usize {
        match self {
            Luminance => 1,
            RedGreenBlue => 3,
            Indexed => 1,
            LuminanceAlpha => 2,
            RedGreenBlueAlpha => 4,
        }
    }
}

impl From<ColorType> for u8 {
    fn from(val: ColorType) -> Self {
        val as u8
    }
}
