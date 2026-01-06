use std::any::type_name;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("{0}{1}")]
pub(crate) struct WrappedError(&'static str, String);

impl WrappedError {
    pub fn new(msg: &'static str, err: impl Debug) -> Self {
        Self(msg, format!("{err:?}"))
    }
}

pub(crate) trait DebugResultExt {
    type Ok;
    fn wrap(self) -> Result<Self::Ok, WrappedError>;
}

impl<T, E: Debug + Any> DebugResultExt for Result<T, E> {
    type Ok = T;
    fn wrap(self) -> Result<T, WrappedError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let prefix = if err.type_id() == TypeId::of::<windows_core::Error>() {
                    "windows_core::"
                } else {
                    let prefix = type_name::<E>();
                    &prefix[..prefix.rfind("::").map(|x| x + 2).unwrap_or(prefix.len())]
                };
                Err(WrappedError::new(prefix, err))
            },
        }
    }
}
