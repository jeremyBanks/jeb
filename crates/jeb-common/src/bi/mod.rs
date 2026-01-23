//! Bijections between numeric types.
pub mod floating;
pub mod hilbert;
pub mod scatter_square;
pub mod scatter_triangle;
pub mod signedness;
pub mod spiral_square;
pub mod spiral_triangle;
pub mod zig_zag;
pub use {
    floating::floating,
    hilbert::hilbert,
    scatter_square::scatter_square,
    scatter_triangle::scatter_triangle,
    signedness::signedness,
    spiral_square::spiral_square,
    spiral_triangle::spiral_triangle,
    z_order::z_order,
    zig_zag::zig_zag,
};

mod test;
pub mod z_order;
