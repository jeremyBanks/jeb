//! Built-in color palettes

pub mod crameri;
pub mod mappings;
pub mod oceanic;
pub mod singles;
pub mod viridis;

pub const RGB_256_COLOR_PALETTE_SIZE: usize = 768;

/// All included [`EightBit`][crate::EightBit]
/// color maps.
pub static ALL: &[&[u8]] = &[];

/// All included [`EightBit`][crate::EightBit]
/// color maps that might be
/// categorized as "sequential".
///
/// ```text
///         AA
///       AAAA
///     AAAAAA
///   AAAAAAAA
/// AAAAAAAAAA
/// ```
pub static ALL_SEQUENTIAL: &[&[u8]] = &[
    oceanic::AMP,
    oceanic::ICE,
    oceanic::OXY,
    crameri::BUDA,
    crameri::NUUK,
    crameri::OSLO,
    oceanic::DEEP,
    oceanic::RAIN,
    crameri::ACTON,
    crameri::DAVOS,
    crameri::DEVON,
    crameri::IMOLA,
    crameri::LAPAZ,
    crameri::TOKYO,
    crameri::TURKU,
    oceanic::ALGAE,
    oceanic::DENSE,
    oceanic::SOLAR,
    oceanic::SPEED,
    oceanic::TEMPO,
    singles::TURBO,
    viridis::MAGMA,
    crameri::BAMAKO,
    crameri::BATLOW,
    crameri::BILBAO,
    crameri::HAWAII,
    oceanic::HALINE,
    oceanic::MATTER,
    oceanic::TURBID,
    viridis::PLASMA,
    crameri::LAJOLLA,
    oceanic::THERMAL,
    singles::CIVIDIS,
    viridis::INFERNO,
    viridis::VIRIDIS,
    crameri::BATLOW_K,
    crameri::BATLOW_W,
];

/// All included [`EightBit`][crate::EightBit]
/// color maps that might be
/// categorized as "diverging".
///
/// ```text
/// A        B
/// AA      BB
/// AAA    BBB
/// AAAA  BBBB
/// AAAAABBBBB
/// ```
pub static ALL_DIVERGING: &[&[u8]] = &[
    crameri::BAM,
    crameri::VIK,
    crameri::BROC,
    crameri::CORK,
    crameri::ROMA,
    oceanic::CURL,
    oceanic::DIFF,
    oceanic::TARN,
    oceanic::DELTA,
    crameri::BERLIN,
    crameri::LISBON,
    crameri::TOFINO,
    crameri::VANIMO,
    oceanic::BALANCE,
];

/// All included [`EightBit`][crate::EightBit]
/// color maps that might be
/// categorized as "dual-sequential" (or "multi-sequential").
///
/// ```text
///     A    B
///    AA   BB
///   AAA  BBB
///  AAAA BBBB
/// AAAAABBBBB
/// ```
pub static ALL_DUAL_SEQUENTIAL: &[&[u8]] = &[
    oceanic::TOPO,
    crameri::FES,
    crameri::OLERON,
    crameri::BUKAVU,
];

/// All included [`EightBit`][crate::EightBit]
/// color maps that might be
/// categorized as "cyclic".
///
/// ```text
///     BB
///    ABBA
///   AABBAA
///  AAABBAAA
/// AAAABBAAAA
/// ```
pub static ALL_CYCLIC: &[&[u8]] = &[
    crameri::BAM_O,
    crameri::VIK_O,
    oceanic::PHASE,
    crameri::BROC_O,
    crameri::CORK_O,
    crameri::ROMA_O,
];
