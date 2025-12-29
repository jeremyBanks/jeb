pub mod floating;
pub mod hilbert;
pub mod scatter_square;
pub mod signedness;
pub mod zig_zag;


// TODO: add hamiltonian_hilbert, which uses the looped/Hamiltonian version of
// the Hilbert curve to also preserve locality when looking at the input as
// being in a modular space. XXX: oh that's just called a moore curve?
// TODO: add spiral_square, which goes in rings instead of scattering, following
// the general quadrant order as we use when measuring angle around a circle.

pub use {
    floating::floating,
    hilbert::hilbert,
    scatter_square::scatter_square,
    signedness::signedness,
    zig_zag::zig_zag,
};
