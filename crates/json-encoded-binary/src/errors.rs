#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Panic {}

impl<Error: ::core::fmt::Debug> From<Error> for Panic {
    fn from(error: Error) -> Self {
        panic!("{error:#?}");
    }
}

impl ::core::fmt::Display for Panic {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        unreachable!()
    }
}
