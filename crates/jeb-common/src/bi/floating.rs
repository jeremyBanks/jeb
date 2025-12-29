#![doc = description!()]
macro_rules! description {
    () => {
        r#"
Bijection between floating point and unsigned integer types which preserves
the ordering of values (as defined by IEEE 754-2008).
        "#
    };
}
use description;


#[doc = description!()]
pub fn floating<T: Floating>(value: T) -> T::Out {
    unimplemented!()
}

trait Floating {
    type Out;
}
