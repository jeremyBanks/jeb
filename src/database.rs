use std::future::Future;

use serde_json::{Value as Json, json};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use bincode::Options;

pub trait Model: Serialize + DeserializeOwned {
    fn name() -> &'static str;
    fn key(&self) -> String;
}

#[derive(Clone)]
struct Connection {
    pub sled: sled::Db,
}

impl Connection {
    pub fn open(path: &str) -> eyre::Result<Self> {
        Ok(Connection {
            sled: sled::open(path)?,
        })
    }

    pub fn table<ModelType: Model>(&self) -> eyre::Result<Tree<ModelType>> {
        Ok(Tree {
            tree: self.sled.open_tree(ModelType::name())?,
            _type: Default::default(),
        })
    }
}

#[derive(Clone)]
pub struct Tree<ModelType: Model> {
    pub tree: sled::Tree,
    pub _type: std::marker::PhantomData<ModelType>,
}

impl<ModelType: Model> Tree<ModelType> {
    #[tracing::instrument(skip(self, model), level = "debug")]
    pub fn insert(&self, model: &ModelType) -> eyre::Result<()> {
        let key = model.key();
        let value = bincode::options().with_big_endian().serialize(model)?;
        let had_existing = self.tree.insert(&key, value)?.is_some();
        tracing::debug!(had_existing, "inserted {}", &key);
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn get(&self, key: &str) -> eyre::Result<Option<ModelType>> {
        Ok(match self.tree.get(key)? {
            Some(bytes) => {
                tracing::debug!(key, "found existing value");
                let value: ModelType = bincode::options().with_big_endian().deserialize(&bytes)?;
                assert_eq!(key, value.key());
                Some(value)
            }
            None => {
                tracing::debug!(key, "existing value not found");
                None
            }
        })
    }

    #[tracing::instrument(skip(self, f), level = "debug")]
    pub async fn get_or_insert_with_async<T>(&self, key: &str, f: impl FnOnce() -> T) -> eyre::Result<ModelType> where
        T: Future<Output=ModelType>,
     {
        match self.get(key) {
            Ok(Some(existing)) => {
                tracing::debug!(key, "found existing value");
                return Ok(existing);
            }
            Ok(None) => {
                tracing::debug!(key, "existing value not found");
            }
            Err(error) => {
                tracing::warn!(key, "ignoring unreadable existing value: {}", error);
            }
        }

        let value = f().await;
        assert_eq!(key, value.key());
        self.insert(&value)?;
        Ok(value)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSearchKey {
    name_contains: String,

}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSearch {
    pub key: GameSearchKey,
    pub response: Json,
}

#[allow(unused)]
async fn test(k: GameSearchKey, t: Tree<GameSearch>) {
    t.get_or_insert_with_async("some_key", || async {
        GameSearch {
            key: k.clone(),
            response: json!([])
        }
    }).await;
}

impl Model for GameSearch {
    fn name() -> &'static str {
        "GameSearch"
    }

    fn key(&self) -> String {
        self.key.name_contains.clone()
    }
}
