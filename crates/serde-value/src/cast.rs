//! Type casting via serde serialization/deserialization.
//!
//! This module provides [`try_cast`], which converts between types that implement
//! serde's `Serialize` and `Deserialize` traits by going through [`Value`] as an
//! intermediate representation.

use crate::{from_value, to_value, Error};
use serde::{de::DeserializeOwned, Serialize};

/// Cast a value of one type to another via serde.
///
/// This function serializes the input value to [`Value`], then deserializes it
/// as the target type. This allows converting between types that have compatible
/// serde representations.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use serde_value::try_cast;
///
/// #[derive(Serialize)]
/// struct UserV1 { name: String, age: u32 }
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct UserV2 { name: String, age: u32 }
///
/// let v1 = UserV1 { name: "Alice".into(), age: 30 };
/// let v2: UserV2 = try_cast(&v1).unwrap();
/// assert_eq!(v2.name, "Alice");
/// assert_eq!(v2.age, 30);
/// ```
///
/// # Errors
///
/// Returns an error if serialization or deserialization fails. This can happen
/// if the source and target types have incompatible serde representations.
pub fn try_cast<T: Serialize, U: DeserializeOwned>(value: &T) -> Result<U, Error> {
    let intermediate = to_value(value)?;
    from_value(intermediate)
}
