use std::{borrow::Cow, fmt::Debug, future::Future};

use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value as Json};

// I want to be able to get records by primary key.
// Which is to say, by key prefix?
// How can you expose that in a Rust API? You probably can't.
// These aren't primary, they're composite.
// Don't rely on bincoding?
// Or maybe do. Who knows? I don't.

fn bincoder() -> impl bincode::Options {
    // bincoding options to help maintain some sort orderings after serialization.
    bincode::options().with_big_endian().with_fixint_encoding()
}

pub struct RowVersion<RowType: Row> {
    /// Sled-generated ID when this row is inserted.
    version_id: u64,
    /// The actual contents of this row.
    row: RowType,
}

// You can do this without putting it in the code, eh?

pub trait Row: Serialize + DeserializeOwned {
    type PrimaryKey: serde::Serialize;

    /// Determines row identity across versions.
    ///
    /// May be ommitted to use value identity (by way of a cryptographic hash function).
    fn PrimaryKey(&self) -> Self::PrimaryKey {
        let serialized = bincoder().serialize(&self).expect("failed to bincode for primary_key");
        blake3::hash(&serialized).as_bytes()[..16].to_vec();
    }

    /// Determines the priority of this version. When looking up a row by primary key, they are
    /// ranked by priority, then by time. For example, this could be used to return versions
    /// containing successful values over this returning containing failed values. If ommitted,
    /// rows will only be sorted chronologically by version_id.
    fn PriorityKey(&self) -> dyn serde::Serialize {}
}

// Getset on top of that?

pub trait PrimaryKey: Serialize + DeserializeOwned + Clone + Debug {

}


pub trait Rowa: Serialize + DeserializeOwned {
    const TABLE_NAME: &'static str;

    type PrimaryKey: PrimaryKey;

    fn primary_key(&self) -> Self::PrimaryKey {
        let serialized = bincoder().serialize(&self).expect("failed to bincode for primary_key");

        let hashed = blake3::hash(&serialized).as_bytes()[..16].to_vec();

        unimplemented!()
    }
}

#[derive(Serialize, Deserialize)]

struct StadiaApiCall {
    method_id: String,
    arguments: Vec<Json>,
    result: StadiaApiCallResult,
    sled_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]

enum StadiaApiCallResult {
    /// Unknown; this call was not completed or the result was not captured.
    Unknown,
    /// The call failed for out-of-band reasons (i.e. network error, unexpected
    /// response format).
    Unable { error_message: String },
    /// The call failed with an in-band error response value.
    Error { value: Json },
    /// The call succeeded with a successful response value.
    Success { value: Json },
}

// impl Table for StadiaApiRequest {
//     const NAME: &'static str = "stadia_api_request";
// }

// #[derive(Clone)]
// struct Database {
//     sled: sled::Db,
// }

// impl Database {
//     pub fn open(path: &str) -> eyre::Result<Self> {
//         Ok(Database {
//             sled: sled::open(path)?,
//         })
//     }

//     pub fn model<ModelType: Model>(&self) -> eyre::Result<Tree<ModelType>> {
//         Ok(Tree {
//             tree: self.sled.open_tree(ModelType::name())?,
//             _type: Default::default(),
//         })
//     }
// }

// #[derive(Clone)]
// pub struct Tree<ModelType: Model> {
//     pub tree: sled::Tree,
//     pub _type: std::marker::PhantomData<ModelType>,
// }

// impl<ModelType: Model> Tree<ModelType> {
//     #[tracing::instrument(skip(self, model), level = "debug")]
//     pub fn insert(&self, model: &ModelType) -> eyre::Result<()> {
//         let key = model.key();
//         let value = bincode::options().with_big_endian().serialize(model)?;
//         let had_existing = self.tree.insert(&key, value)?.is_some();
//         tracing::debug!(had_existing, "inserted {}", &key);
//         Ok(())
//     }

//     #[tracing::instrument(skip(self), level = "debug")]
//     pub fn get(&self, key: &str) -> eyre::Result<Option<ModelType>> {
//         Ok(match self.tree.get(key)? {
//             Some(bytes) => {
//                 tracing::debug!(key, "found existing value");
//                 let value: ModelType =
// bincode::options().with_big_endian().deserialize(&bytes)?;
// assert_eq!(key, value.key());                 Some(value)
//             }
//             None => {
//                 tracing::debug!(key, "existing value not found");
//                 None
//             }
//         })
//     }

//     #[tracing::instrument(skip(self, f), level = "debug")]
//     pub async fn get_or_insert_with_async<T>(
//         &self,
//         key: &str,
//         f: impl FnOnce() -> T,
//     ) -> eyre::Result<ModelType>
//     where
//         T: Future<Output = ModelType>,
//     {
//         match self.get(key) {
//             Ok(Some(existing)) => {
//                 tracing::debug!(key, "found existing value");
//                 return Ok(existing);
//             }
//             Ok(None) => {
//                 tracing::debug!(key, "existing value not found");
//             }
//             Err(error) => {
//                 tracing::warn!(key, "ignoring unreadable existing value: {}",
// error);             }
//         }

//         let value = f().await;
//         assert_eq!(key, value.key());
//         self.insert(&value)?;
//         Ok(value)
//     }
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct GameSearchKey {
//     name_contains: String,
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct GameSearch {
//     pub key: GameSearchKey,
//     pub response: Json,
// }

// #[allow(unused)]
// async fn test(k: GameSearchKey, t: Tree<GameSearch>) {
//     t.get_or_insert_with_async("some_key", || async {
//         GameSearch {
//             key: k.clone(),
//             response: json!([]),
//         }
//     })
//     .await;
// }

// impl Model for GameSearch {
//     fn name() -> &'static str {
//         "GameSearch"
//     }

//     fn key(&self) -> String {
//         self.key.name_contains.clone()
//     }
// }
