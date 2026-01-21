/// Potentially-fallible potentially-lossy conversion trait.
///
/// This trait is meant to be implemented, but rarely to be used directly. Users
/// will typically want to use a simpler trait that delegates to this one.
pub trait TryFromMaybeLossy<T>: Sized {
    /// A warning returned if the conversion is lossy.
    type Warning;

    /// An error returned if the conversion is not possible.
    type Error;

    /// Attempts to convert the value to `Self`, potentially also returning a
    /// `Warning` if the conversion is lossy, or instead returning an `Error` if
    /// the conversion is not possible.
    fn try_from_maybe_lossy(value: T) -> Result<(Self, Option<Self::Warning>), Self::Error>;

    /// `try_from_maybe_lossy` implementation helper function for lossless
    /// results.
    fn lossless(value: Self) -> Result<(Self, Option<Self::Warning>), Self::Error> {
        Ok((value, None))
    }

    /// `try_from_maybe_lossy` implementation helper function for lossy results.
    fn lossy(
        value: Self,
        warning: Self::Warning,
    ) -> Result<(Self, Option<Self::Warning>), Self::Error> {
        Ok((value, Some(warning)))
    }

    /// `try_from_maybe_lossy` implementation helper function for error results.
    fn error(error: Self::Error) -> Result<(Self, Option<Self::Warning>), Self::Error> {
        Err(error)
    }

    /// Attempts to convert the value to `Self`, potentially instead returning a
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

/// Trait identifying the `std::convert::Infallible` type.
pub trait Infallible {
    fn unreachable(self) -> !;
}

impl Infallible for std::convert::Infallible {
    fn unreachable(self) -> ! {
        match self {}
    }
}

/// Potentially-fallible lossless conversion trait (delegates to
/// `TryFromMaybeLossy`).
pub trait TryFromLossless<T>: TryFromMaybeLossy<T> {
    fn try_from_lossless(value: T) -> Result<Self, Result<Self::Warning, Self::Error>> {
        TryFromMaybeLossy::try_from_lossless(value)
    }
}
impl<T, Warning, Error> TryFromLossless<T> for T where
    T: TryFromMaybeLossy<T, Warning = Warning, Error = Error>
{
}

/// Infallible lossless conversion trait (delegates to `TryFromMaybeLossy`).
pub trait FromLossless<T>: TryFromMaybeLossy<T>
where
    Self::Warning: Infallible,
    Self::Error: Infallible,
{
    fn from_lossless(value: T) -> Self {
        TryFromMaybeLossy::from_lossless(value)
    }
}
impl<T> FromLossless<T> for T where T: TryFromMaybeLossy<T, Warning: Infallible, Error: Infallible> {}

/// Potentially-fallible lossy conversion trait (delegates to
/// `TryFromMaybeLossy`).
pub trait TryFromLossy<T>: TryFromMaybeLossy<T> {
    fn try_from_lossy(value: T) -> Result<Self, Self::Error> {
        TryFromMaybeLossy::try_from_lossy(value)
    }
}
impl<T, Warning, Error> TryFromLossy<T> for T where
    T: TryFromMaybeLossy<T, Warning = Warning, Error = Error>
{
}

/// Infallible lossy conversion trait (delegates to `TryFromMaybeLossy`).
pub trait FromLossy<T>: TryFromLossy<T>
where
    Self::Error: Infallible,
{
    fn try_from_lossy(value: T) -> Result<Self, Self::Error> {
        TryFromLossy::try_from_lossy(value)
    }
}
impl<T, Warning, Error> FromLossy<T> for T
where
    T: TryFromLossy<T, Warning = Warning, Error = Error>,
    Error: Infallible,
{
}

fn main() {}
