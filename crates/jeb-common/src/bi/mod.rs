//! Bijections between numeric types.

pub mod floating;
pub mod hilbert;
pub mod moore;
pub mod scatter_square;
pub mod signedness;
pub mod spiral_square;
pub mod zig_zag;


// TODO: moore curve's closed-loop property (index 0 and MAX adjacent) is not
// working correctly - the algorithm needs fixing. The basic locality between
// adjacent indices works, but the first-to-last wrap-around does not.

pub use {
    floating::floating,
    hilbert::hilbert,
    moore::moore,
    scatter_square::scatter_square,
    signedness::signedness,
    spiral_square::spiral_square,
    zig_zag::zig_zag,
};
