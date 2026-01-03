#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Panic {}
#[expect(clippy::fallible_impl_from)]
impl<Error: ::core::fmt::Debug> From<Error> for Panic {
    fn from(error: Error) -> Self {
        panic!("{error:#?}");
    }
}
impl ::core::fmt::Display for Panic {
    fn fmt(&self, _f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        unreachable!()
    }
}
