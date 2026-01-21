/// Implementation helper trait for potentially-fallible potentially-lossy
/// conversions. Implementors should implement this crate, while users will
/// instead use one of the narrower delegating traits: `TryFromMaybeLossy`,
/// `TryFromLossless`, `FromLossless`, `TryFromLossy`, or `FromLossy`.
pub trait TryFromMaybeLossyImpl<T>: Sized {
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
            Ok((_value, Some(warning))) => Err(Ok(warning)),
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

// Potentially-fallible potentially-lossy conversion trait (delegates to
// `TryFromMaybeLossyImpl`).
pub trait TryFromMaybeLossy<T>: TryFromMaybeLossyImpl<T> {
    fn try_from_maybe_lossy(value: T) -> Result<(Self, Option<Self::Warning>), Self::Error> {
        TryFromMaybeLossyImpl::try_from_maybe_lossy(value)
    }
}
impl<T, Warning, Error> TryFromMaybeLossy<T> for T where
    T: TryFromMaybeLossyImpl<T, Warning = Warning, Error = Error>
{
}

/// Trait identifying the `std::convert::Infallible` type.
pub trait Infallible {
    fn unreachable(self) -> !;

    fn any<T>(self) -> T;
}

impl Infallible for std::convert::Infallible {
    fn unreachable(self) -> ! {
        match self {}
    }

    fn any<T>(self) -> T {
        self.unreachable()
    }
}

/// Potentially-fallible lossless conversion trait (delegates to
/// `TryFromMaybeLossy`).
pub trait TryFromLossless<T>: TryFromMaybeLossyImpl<T> {
    fn try_from_lossless(value: T) -> Result<Self, Result<Self::Warning, Self::Error>> {
        TryFromMaybeLossyImpl::try_from_lossless(value)
    }
}
impl<T, Warning, Error> TryFromLossless<T> for T where
    T: TryFromMaybeLossyImpl<T, Warning = Warning, Error = Error>
{
}

/// Infallible lossless conversion trait (delegates to `TryFromMaybeLossy`).
pub trait FromLossless<T>: TryFromMaybeLossyImpl<T>
where
    Self::Warning: Infallible,
    Self::Error: Infallible,
{
    fn from_lossless(value: T) -> Self {
        TryFromMaybeLossyImpl::from_lossless(value)
    }
}
impl<T> FromLossless<T> for T where
    T: TryFromMaybeLossyImpl<T, Warning: Infallible, Error: Infallible>
{
}

/// Potentially-fallible lossy conversion trait (delegates to
/// `TryFromMaybeLossy`).
pub trait TryFromLossy<T>: TryFromMaybeLossyImpl<T> {
    fn try_from_lossy(value: T) -> Result<Self, Self::Error> {
        TryFromMaybeLossyImpl::try_from_lossy(value)
    }
}
impl<T, Warning, Error> TryFromLossy<T> for T where
    T: TryFromMaybeLossyImpl<T, Warning = Warning, Error = Error>
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

/// Attempts to convert the value to the first type that succeeds losslessly if
/// any do, otherwise returns the first type that fails losslessly and its
/// associated warning, otherwise returns the first error.
///
/// Requires that each `$intermediary` type implements
/// `TryFromMaybeLossyImpl<T>` where `T` is the type of the original value, and
/// that each of those implementations uses the same `Warning` type and the same
/// `Error` type, and that the `$target` type (which may be explicit or inferred
/// ) implements `From<T>` for each type `T` in the `$intermediary` types.
macro_rules! try_from_maybe_lossy_via {
    ($value:expr => $($intermediary:ty)|+ $( => $target:ty)?) => {
        {
            use $crate::TryFromMaybeLossyImpl;

            let value = $value;

            let first_lossless $(: Option<$target>)? = None;
            let first_lossy $(: (Option<$target>, _))? = None;
            let first_error = None;

            $(
                let value = $type::try_from($value)?;
                if first_lossless.is_none() {
                    first_lossless = Some(value);
                } else if first_lossy.is_none() {
                    first_lossy = Some(value);
                } else {
                    first_error = Some(value);
                }
            )+

            // XXX: this logic is wrong because we need to pull it inside, of
            // course. or rather, up above. this is a mess but the gist is
            // valid.


            if let Some(first_lossless) = first_lossless {
                let value:  = first_lossless.into();
                $(
                    let value: $target = value;
                )?
                warning = None;
                Ok((value, warning))
            } else if let Some(first_lossy) = first_lossy {
                value = first_lossy.0.into();
                warning = first_lossy.1;
            } else let Some(first_error) = first_error {
                Err(first_error);
            };

            (value, warning)
        }
    }
}

fn main() {}
