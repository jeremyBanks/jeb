#![allow(unused)]

/// Potentially-fallible potentially-lossy conversion trait.
trait TryFromMaybeLossy<T>: Sized {
    /// A warning returned if the conversion is lossy.
    type Warning;

    /// An error returned if the conversion is not possible.
    type Error;

    /// Attempts to convert the value to `Self`, potentially also returning a
    /// `Warning` if the conversion is lossy, or instead returning an `Error` if
    /// the conversion is not possible.
    fn try_from_maybe_lossy(value: T) -> Result<(Self, Option<Self::Warning>), Self::Error>;

    /// Attempts to convert the value to Self, potentially instead returning a
    /// `Warning` if the conversion is lossy or an `Error` if the conversion is
    /// not possible.
    fn try_from_lossless(value: T) -> Result<Self, Result<Self::Warning, Self::Error>> {
        match Self::try_from_maybe_lossy(value) {
            Ok((value, None)) => Ok(value),
            Ok((value, Some(warning))) => Err(Ok(warning)),
            Err(error) => Err(Err(error)),
        }
    }

    /// Attempts to convert the value to `Self`, ignoring warnings about
    /// lossiness, instead returning an `Error` if the conversion is not
    /// possible.
    fn try_from_lossy(value: T) -> Result<Self, Self::Error> {
        match Self::try_from_maybe_lossy(value) {
            Ok((value, _maybe_warning)) => Ok(value),
            Err(error) => Err(error),
        }
    }

    /// Converts the value to `Self`, ignoring warnings about lossiness
    /// (infallible).
    fn from_lossy(value: T) -> Self
    where
        Self::Error: Infallible,
    {
        match Self::try_from_maybe_lossy(value) {
            Ok((value, None)) => value,
            Ok((value, Some(_warning))) => value,
            Err(error) => error.unreachable(),
        }
    }

    /// Converts the value to `Self` losslessly (infallible).
    fn from_lossless(value: T) -> Self
    where
        Self::Warning: Infallible,
        Self::Error: Infallible,
    {
        match Self::try_from_lossless(value) {
            Ok(value) => value,
            Err(Ok(warning)) => warning.unreachable(),
            Err(Err(error)) => error.unreachable(),
        }
    }
}

/// Trait for the `std::from::Infallible` type, used to enable the
/// `from<T>::from` method for lossless infallible conversions and
/// `from<T>::from_lossy` method for potentially-lossy infallible
/// conversions.
pub trait Infallible {
    fn unreachable(self) -> !;
}

impl Infallible for std::convert::Infallible {
    fn unreachable(self) -> ! {
        match self {}
    }
}

fn main() {}
