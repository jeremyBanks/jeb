use std::any::Any;
use std::fmt;

use derive_more::AsRef;
use derive_more::Deref;
use derive_more::From;
use derive_more::Into;
use serde::de::Visitor;
use serde::ser::SerializeTuple;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::panic;

/// Wrapper type for bytes that will be serialized/deserialized without any
/// form of terminator/delimiter. As many bytes will be read as the
/// (de)serializer will allow, stopping at a None or Err.
#[derive(Clone, Default, PartialEq, PartialOrd, Eq, Ord, Debug, AsRef, From, Into, Deref)]
pub struct UnterminatedBytes(Vec<u8>);

impl Serialize for UnterminatedBytes {
    fn serialize<Ser: Serializer>(&self, ser: Ser) -> Result<Ser::Ok, Ser::Error> {
        let bytes = self.as_ref();
        let mut tuple = ser.serialize_tuple(bytes.len())?;
        for byte in bytes {
            tuple.serialize_element(byte)?;
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for UnterminatedBytes {
    fn deserialize<De: Deserializer<'de>>(de: De) -> Result<Self, De::Error> {
        let mut bytes = Vec::<u8>::new();
        de.deserialize_tuple(usize::MAX, BytesVisitor(&mut bytes))?;
        Ok(UnterminatedBytes(bytes))
    }
}

struct BytesVisitor<'a>(&'a mut Vec<u8>);
impl<'de, 'a> Visitor<'de> for BytesVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a byte")
    }

    fn visit_u8<Err>(self, byte: u8) -> Result<Self::Value, Err> {
        self.0.push(byte);
        Ok(())
    }

    fn visit_seq<Seq>(self, mut seq: Seq) -> Result<Self::Value, Seq::Error>
    where Seq: serde::de::SeqAccess<'de> {
        loop {
            match seq.next_element::<u8>() {
                Ok(next) => match next {
                    Some(byte) => self.0.push(byte),
                    None => break Ok(()),
                },
                Err(err) => {
                    break Ok(());
                },
            }
        }
    }
}

#[cfg(test)]
#[test]
fn unterminated_bytes() -> Result<(), panic> {
    use postcard;

    let byte: u8 = 0xDD;
    let number: u64 = 1;
    let string = "hello.";
    let bytes = UnterminatedBytes(b"hello world!".to_vec());

    {
        let as_terminator = (byte, number, string.to_string(), bytes.clone());
        let serialized = postcard::to_stdvec(&as_terminator)?;
        assert_eq!(b"\xDD\x01\x06hello.hello world!", serialized.as_slice());
        let deserialized: (u8, u64, String, UnterminatedBytes) = postcard::from_bytes(&serialized)?;
        assert_eq!(as_terminator, deserialized);
    }

    {
        let as_head = (bytes, number, byte, string.to_string());
        let serialized = postcard::to_stdvec(&as_head)?;
        assert_eq!(b"hello world!\x01\xDD\x06hello.", serialized.as_slice());
        let deserialized: Result<(UnterminatedBytes, u64, u8, String), _> =
            postcard::from_bytes(&serialized);
        assert!(deserialized.is_err());
    }

    Ok(())
}

#[cfg(test)]
#[test]
fn unterminated_bytes_in_json() -> Result<(), panic> {
    use serde_json;

    let byte: u8 = 221;
    let number: u64 = 1;
    let string = "hello.";
    let bytes = UnterminatedBytes(b"hello world!".to_vec());

    {
        let as_terminator = (byte, number, string.to_string(), bytes.clone());
        let serialized = serde_json::to_string(&as_terminator)?;
        assert_eq!(
            r#"[221,1,"hello.",[104,101,108,108,111,32,119,111,114,108,100,33]]"#,
            serialized
        );
        let deserialized: (u8, u64, String, UnterminatedBytes) = serde_json::from_str(&serialized)?;
        assert_eq!(as_terminator, deserialized);
    }

    {
        let as_head = (bytes, number, byte, string.to_string());
        let serialized = serde_json::to_string(&as_head)?;
        assert_eq!(
            r#"[[104,101,108,108,111,32,119,111,114,108,100,33],1,221,"hello."]"#,
            serialized
        );
        let deserialized: (UnterminatedBytes, u64, u8, String) = serde_json::from_str(&serialized)?;
        assert_eq!(as_head, deserialized); // this works for JSON, but it's not
                                           // guaraunteed
    }

    Ok(())
}
