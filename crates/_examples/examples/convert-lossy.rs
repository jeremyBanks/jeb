use core::{
    convert::Infallible as Never,
    fmt::Debug,
    marker::PhantomData,
};

/// Type level `Eq` operation.
#[expect(private_bounds)]
pub trait Eq<T>: InnerEq<T> {}
impl<T> InnerEq<T> for T {}
impl<T> Eq<T> for T where T: InnerEq<T> {}
trait InnerEq<T> {}

/// Type-level `bool` value.
#[expect(private_bounds)]
pub trait Bool: InnerBool {}
impl InnerBool for True {}
impl InnerBool for False {}
pub enum True {}
pub enum False {}
trait InnerBool {}



pub trait ImplConversionsFrom<Source>: Sized {
    type Supported: Bool;
    type Warning: Debug;
    type Error: Debug;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConversionPriority {
    SelfReversible,
    ContextuallyReversible,
    SemanticallyEquivalent,
    NotClamped,
    NotTruncated,
}

// XXX: instead of bools, these need to be optional generic error
// types so we can statically exclude them if they're defined
// to be Never.

// like NotSelfReversibleError = () etc by default or something, hah

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConversionResult<Target, Source>
where
    Target: ImplConversionsFrom<Source>,
{
    source: PhantomData<fn(Source)>,

    /// The converted value, or an error if we were unable to produce one.
    value: Result<Target, Target::Error>,

    /// Whether this conversion can be losslessly infallibly converted back to
    /// the source by the inverse operation, with no additional context except
    /// the types `Source` and `Target`.
    self_reversible: bool,

    /// Whether this conversion could hypothetically be losslessly converted
    /// back to the source given some additional context/metadata such as a
    /// schema or discriminants or type hints, regardless of whether we actually
    /// support that. This usually implies that if the source if of a consistent
    /// non-pathological format, the content will be preserved in full detail
    /// (even if restoring the original structure is more complicated).
    contextually_reversible: bool,

    /// Whether value and its type still have roughly the same semantic meaning,
    /// even if it's represented differently. Example: different numeric types
    /// representing exactly the same value are semantically equivalent, but
    /// string or byte representations of those values are not. This doesn't
    /// preclude clamping or rounding or truncation if it's done in a way that's
    /// more-or-less respectful of the semantic value. (However, an overflowing
    /// value that wraps around is _not_ semantically equivalent — we're not
    /// thinking in modular arithmetic.)
    semantically_equivalent: bool,

    /// Whether the value was clamped to a minimum or maximum value (which may
    /// be finite or infinite) due to being outside the supported range.
    clamped: bool,

    /// Whether the value was truncated or rounded to a lower precision or to
    /// align with a different value (but not due to going fully out of range),
    /// or due to the value being too large (in data size, not magnitude).
    truncated: bool,
}

impl<Target, Source> ConversionResult<Target, Source>
where
    Target: ImplConversionsFrom<Source>,
{
    pub fn with_error(error: Target::Error) -> Self {
        Self {
            value: Err(error),
            source: PhantomData,
            self_reversible: false,
            contextually_reversible: false,
            semantically_equivalent: false,
            clamped: false,
            truncated: false,
        }
    }

    pub fn error(&self) -> Option<&Target::Error> {
        self.value.as_ref().err()
    }

    pub fn with_identical(value: Target) -> Self {
        Self {
            value: Ok(value),
            source: PhantomData,
            self_reversible: true,
            contextually_reversible: true,
            semantically_equivalent: true,
            clamped: false,
            truncated: false,
        }
    }

    pub fn is_identical(&self) -> bool {
        self.value.is_ok()
            && self.self_reversible
            && self.contextually_reversible
            && self.semantically_equivalent
            && !self.clamped
            && !self.truncated
    }

    pub fn is_self_reversible(&self) -> bool {
        self.value.is_ok() && self.self_reversible
    }

    pub fn is_contextually_reversible(&self) -> bool {
        self.value.is_ok() && self.contextually_reversible
    }

    pub fn is_semantically_equivalent(&self) -> bool {
        self.value.is_ok() && self.semantically_equivalent
    }

    pub fn with_reversible_opaque(value: Target) -> Self {
        Self {
            value: Ok(value),
            source: PhantomData,
            self_reversible: true,
            contextually_reversible: true,
            semantically_equivalent: false,
            clamped: false,
            truncated: false,
        }
    }

    pub fn with_contextually_reversible_opaque(value: Target) -> Self {
        Self {
            value: Ok(value),
            source: PhantomData,
            self_reversible: false,
            contextually_reversible: true,
            semantically_equivalent: false,
            clamped: false,
            truncated: false,
        }
    }
}



// XXX: Okay I think our internal type can actually just bite the bullet and be
// very precise about what it's returning, since we'll actually expose cleaner
// external interfaces.

#[allow(unused)]
mod thinking {
    use core::convert::Infallible as Never;

    enum ConversionResult<T, TypeLossWarning = Never, ValueLossWarning = Never, FatalError = Never> {
        Lossless(T),
        LossyType(T, TypeLossWarning),
        LossyValue(T, ValueLossWarning),
        Error(FatalError),
    }
}

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
