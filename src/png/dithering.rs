use {
    derive_more::{
        Add, AddAssign, Constructor, Div, DivAssign, Mul, MulAssign, Neg, Product, Sub, SubAssign,
        Sum,
    },
    oklab,
    rgb::{RGB8, RGBA8},
    std::hash::{Hash, Hasher},
};

fn dither(palette: Vec<RGB8>, pixels: Vec<RGB8>, width: usize) -> Vec<usize> {
    let output = Vec::with_capacity(pixels.len());

    let ok = Color::default();

    let red = Color::new_rgb(0xFF, 0x00, 0x00);

    let diff = ok - red;

    let mag = diff.magnitude();

    let a = diff / mag;

    // for pixel in pixels {
    //     let oklab: Oklab = srgb_to_oklab(pixel).into();
    //     let (v, p) = palette.into_iter().map(|color|(
    //         color,
    //         Oklab::dif&srgb_to_oklab(color).into(), &oklab)
    //     )).min_by(|a, b|->std::cmp::Ordering{
    //         a.1.partial_cmp(&b.1).unwrap()
    //     }).unwrap();

    //     // return
    // }

    output
}

/// Roughly perceptual Oklab color with alpha.
#[derive(
    Copy,
    Clone,
    Debug,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Default,
    Sum,
    Neg,
)]
pub struct Color {
    /// +1.0 white to -1.0 black
    wb: f32,
    /// +1.0 green to -1.0 red
    rg: f32,
    /// +1.0 blue to -1.0 yellow
    by: f32,
    /// ±0.0 transparent to ±1.0 opaque
    ot: f32,
}

impl Color {
    pub fn new_lab(l: f32, a: f32, b: f32) -> Color {
        Color {
            wb: l,
            rg: a,
            by: b,
            ot: 0.,
        }
    }

    pub fn new_labo(l: f32, a: f32, b: f32, t: f32) -> Color {
        Color {
            wb: l,
            rg: a,
            by: b,
            ot: t,
        }
    }

    pub fn new_rgb(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgb(RGB8::new(r, g, b))
    }

    pub fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::from_rgba(RGBA8::new(r, g, b, a))
    }

    pub fn diff(self, other: Color) -> f32 {
        (self - other).magnitude()
    }

    pub fn simm(self, other: Color) -> f32 {
        1.0 - self.diff(other)
    }

    pub fn magnitude(self) -> f32 {
        (self.wb * self.wb + self.rg * self.rg + self.by * self.by + 3. * self.ot * self.ot).sqrt()
    }

    pub fn l(self) -> f32 {
        self.wb
    }
    pub fn a(self) -> f32 {
        self.rg
    }
    pub fn b(self) -> f32 {
        self.by
    }
    pub fn o(self) -> f32 {
        self.ot
    }

    pub fn from_rgb(rgb: RGB8) -> Self {
        oklab::srgb_to_oklab(rgb).into()
    }

    pub fn from_rgba(rgba: RGBA8) -> Self {
        let rgb = rgba.rgb();
        let mut new: Color = oklab::srgb_to_oklab(rgb).into();
        new.ot = (rgba.a as f32) / 255.;
        new
    }

    pub fn rgb(self) -> RGB8 {
        oklab::oklab_to_srgb(self.into())
    }

    pub fn rgba(self) -> RGBA8 {
        oklab::oklab_to_srgb(self.into()).alpha(self.alpha())
    }

    pub fn red(self) -> u8 {
        self.rgb().r
    }

    pub fn green(self) -> u8 {
        self.rgb().g
    }

    pub fn blue(self) -> u8 {
        self.rgb().b
    }

    pub fn alpha(self) -> u8 {
        // Note that alpha is linear/non-gamma adjusted, while our
        // .r()/.g()/.b() methods use the sRGB gamma curve.
        ((1.0 - self.ot.abs()) * 255.0).round() as u8
    }
}

impl From<oklab::Oklab> for Color {
    fn from(color: oklab::Oklab) -> Self {
        Color {
            wb: color.l,
            rg: color.a,
            by: color.b,
            ot: 0.,
        }
    }
}

impl From<Color> for oklab::Oklab {
    fn from(color: Color) -> Self {
        oklab::Oklab {
            l: color.wb,
            a: color.rg,
            b: color.by,
        }
    }
}

impl From<RGB8> for Color {
    fn from(color: RGB8) -> Self {
        Color::from_rgb(color)
    }
}

impl From<RGBA8> for Color {
    fn from(color: RGBA8) -> Self {
        Color::from_rgba(color)
    }
}

impl From<Color> for RGB8 {
    fn from(color: Color) -> Self {
        color.rgb()
    }
}

impl From<Color> for RGBA8 {
    fn from(color: Color) -> Self {
        color.rgba()
    }
}

impl Ord for Color {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.wb
            .total_cmp(&other.wb)
            .then(self.rg.total_cmp(&other.rg))
            .then(self.by.total_cmp(&other.by))
    }
}

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == core::cmp::Ordering::Equal
    }
}

impl Eq for Color {}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.wb.to_bits().hash(state);
        self.rg.to_bits().hash(state);
        self.by.to_bits().hash(state);
    }
}
