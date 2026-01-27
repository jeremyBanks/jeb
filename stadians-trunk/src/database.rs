use std::{borrow::Cow, fmt::Debug, future::Future};

use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value as Json};

fn bincoder() -> impl bincode::Options {
    // bincoding options to help maintain some sort orderings after serialization.
    bincode::options().with_big_endian().with_fixint_encoding()
}

#[derive(Serialize, Deserialize)]
pub struct VersionedRow<RowType> {
    pub row: RowType,
    pub version_id: u64,
}

impl<RowType: Row> VersionedRow<RowType> {
    fn key(&self) -> (RowType::PrimaryKey, RowType::VersionKey, u64) {
        (
            self.row.primary_key(),
            self.row.version_key(),
            self.version_id,
        )
    }

    fn key_bytes(&self) -> Vec<u8> {
        bincoder()
            .serialize(&self.key())
            .expect("unable to bincode key")
    }
}
pub trait Row: Serialize + DeserializeOwned + 'static {
    const TABLE_NAME: &'static str;

    type PrimaryKey: serde::Serialize;
    type VersionKey: serde::Serialize;

    /// Determines row identity across versions.
    fn primary_key(&self) -> Self::PrimaryKey;
    fn primary_key_bytes(&self) -> Vec<u8> {
        bincoder()
            .serialize(&self.primary_key())
            .expect("unable to bincode primary key")
    }

    /// Determines the indexing/ordering of versions of a row with the same
    /// primary key. If not unique, the tie will be broken by the version_id.
    fn version_key(&self) -> Self::VersionKey;
    fn version_key_bytes(&self) -> Vec<u8> {
        bincoder()
            .serialize(&self.version_key())
            .expect("unable to bincode version key")
    }
}

#[derive(Serialize, Deserialize)]

struct StadiaApiCall {
    method_id: String,
    arguments: Vec<Json>,
    result: Option<Vec<Json>>,
    status: StadiaApiCallStatus,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd)]

enum StadiaApiCallStatus {
    /// This call has been seeded into the database, but not executed.
    Seeded,
    /// This call has been attempted, but we don't know the result.
    Attempted,
    /// This call failed for out-of-band reasons (i.e. network error, unexpected
    /// response format).
    Unable,
    /// The call failed with an in-band error response value.
    Error,
    /// The call succeeded with a successful response value.
    Success,
}

impl Row for StadiaApiCall {
    const TABLE_NAME: &'static str = "stadia_api_call";

    type PrimaryKey = (String, Vec<Json>);
    fn primary_key(&self) -> Self::PrimaryKey {
        (self.method_id.clone(), self.arguments.clone())
    }

    type VersionKey = StadiaApiCallStatus;
    fn version_key(&self) -> Self::VersionKey {
        self.status
    }
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
