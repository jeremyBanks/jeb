pub(crate) enum Panic {}
impl<Error: ::core::fmt::Debug> From<Error> for Panic {
    fn from(error: Error) -> Self {
        panic!("{error:#?}");
    }
}
