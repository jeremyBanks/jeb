use {
    crate::{default, BitDepth, Png, RedGreenBlue},
    bytemuck::bytes_of,
    std::borrow::Cow::{self, Borrowed, Owned},
};

/// A [`Png`] or input that can be converted to one.
pub trait ToPng {
    /// Create or borrow a [`Png`] from this input.
    fn to_png(&self) -> Cow<Png>;
}

impl ToPng for Png {
    /// Returns a reference to this [`Png`].
    fn to_png(&self) -> Cow<Png> {
        Borrowed(self)
    }
}

impl ToPng for [u8] {
    fn to_png(&self) -> Cow<Png> {
        Owned(Png::from_unstructured_bytes(self))
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> ToPng for [[u8; WIDTH]; HEIGHT] {
    /// Create a [`Png`] from a 2D array of `u8` luminance pixels.
    fn to_png(&self) -> Cow<Png> {
        Owned(Png {
            width: WIDTH,
            height: HEIGHT,
            bit_depth: BitDepth::EightBit,
            color_type: RedGreenBlue,
            palette_data: None,
            transparency_data: None,
            pixel_data: bytes_of(self).to_vec(),
        })
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> ToPng for [[[u8; 3]; WIDTH]; HEIGHT] {
    /// Create a [`Png`] from a 2D array of RGB pixel arrays.
    fn to_png(&self) -> Cow<Png> {
        unimplemented!()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> ToPng for [[[u8; 4]; WIDTH]; HEIGHT] {
    /// Create a [`Png`] from a 2D array of RGBA pixel arrays.
    fn to_png(&self) -> Cow<Png> {
        unimplemented!()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> ToPng for [[u32; WIDTH]; HEIGHT] {
    /// Create a [`Png`] from a 2D array of `u32` RGBA pixels.
    fn to_png(&self) -> Cow<Png> {
        unimplemented!()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> ToPng for [[(u8, u8, u8); WIDTH]; HEIGHT] {
    /// Create a [`Png`] from a 2D array of RGB pixel tuples.
    fn to_png(&self) -> Cow<Png> {
        Owned(
            self.map(|row| row.map(|(r, g, b)| [r, g, b]))
                .to_png()
                .into_owned(),
        )
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> ToPng for [[(u8, u8, u8, u8); WIDTH]; HEIGHT] {
    /// Create a [`Png`] from a 2D array of RGBA pixel tuples.
    fn to_png(&self) -> Cow<Png> {
        Owned(
            self.map(|row| row.map(|(r, g, b, a)| [r, g, b, a]))
                .to_png()
                .into_owned(),
        )
    }
}

impl ToPng for str {
    /// Create a [`Png`] from text, by rendering it with a bitmap font.
    fn to_png(&self) -> Cow<Png> {
        unimplemented!()
    }
}

impl ToPng for fn(&mut Png) {
    /// Create a [`Png`] from a function that mutates a [`Png::default`].
    fn to_png(&self) -> Cow<Png> {
        let mut png = default();
        self(&mut png);
        Owned(png)
    }
}

impl ToPng for fn(Png) -> Png {
    /// Create a [`Png`] from a function that mutates a [`Png::default`].
    fn to_png(&self) -> Cow<Png> {
        Owned(self(default()))
    }
}
