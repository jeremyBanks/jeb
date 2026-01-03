use jeb_value::{from_value, to_value};
use serde::{Deserialize, Serialize};

/// Test internally and adjacently tagged enums
/// These use deserialize_any to peek at the structure

#[test]
fn test_internally_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum Message {
        Request { id: u32, data: String },
        Response { id: u32, result: i32 },
        Error { code: u32, message: String },
    }

    let request = Message::Request {
        id: 1,
        data: "hello".to_string(),
    };

    let value = to_value(&request).unwrap();
    let recovered: Message = from_value(value).unwrap();
    assert_eq!(recovered, request);

    let response = Message::Response { id: 2, result: 42 };
    let value = to_value(&response).unwrap();
    let recovered: Message = from_value(value).unwrap();
    assert_eq!(recovered, response);

    let error = Message::Error {
        code: 404,
        message: "Not found".to_string(),
    };
    let value = to_value(&error).unwrap();
    let recovered: Message = from_value(value).unwrap();
    assert_eq!(recovered, error);
}

#[test]
fn test_adjacently_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "t", content = "c")]
    enum Message {
        Request { id: u32, data: String },
        Response { id: u32, result: i32 },
        Unit,
    }

    let request = Message::Request {
        id: 1,
        data: "hello".to_string(),
    };
    let value = to_value(&request).unwrap();
    let recovered: Message = from_value(value).unwrap();
    assert_eq!(recovered, request);

    let unit = Message::Unit;
    let value = to_value(&unit).unwrap();
    let recovered: Message = from_value(value).unwrap();
    assert_eq!(recovered, unit);
}

#[test]
fn test_untagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Data {
        Number(i32),
        Text(String),
        List(Vec<i32>),
    }

    let num = Data::Number(42);
    let value = to_value(&num).unwrap();
    let recovered: Data = from_value(value).unwrap();
    assert_eq!(recovered, num);

    let text = Data::Text("hello".to_string());
    let value = to_value(&text).unwrap();
    let recovered: Data = from_value(value).unwrap();
    assert_eq!(recovered, text);

    let list = Data::List(vec![1, 2, 3]);
    let value = to_value(&list).unwrap();
    let recovered: Data = from_value(value).unwrap();
    assert_eq!(recovered, list);
}

#[test]
fn test_flattened_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Base {
        id: u32,
        name: String,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Extended {
        #[serde(flatten)]
        base: Base,
        extra: String,
    }

    let ext = Extended {
        base: Base {
            id: 1,
            name: "test".to_string(),
        },
        extra: "data".to_string(),
    };

    let value = to_value(&ext).unwrap();
    let recovered: Extended = from_value(value).unwrap();
    assert_eq!(recovered, ext);
}
