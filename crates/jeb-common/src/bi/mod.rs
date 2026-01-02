//! Bijections between numeric types.
pub mod floating;
pub mod hilbert;
pub mod scatter_square;
pub mod signedness;
pub mod spiral_square;
pub mod zig_zag;
pub use {
    floating::floating,
    hilbert::hilbert,
    scatter_square::scatter_square,
    signedness::signedness,
    spiral_square::spiral_square,
    zig_zag::zig_zag,
};
