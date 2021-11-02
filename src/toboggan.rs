use std::future::Future;

use serde_json::{Value as Json, json};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sled;
use bincode;

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

    pub fn tree<ModelType: Model>(&self) -> eyre::Result<Tree<ModelType>> {
        Ok(Tree {
            tree: self.sled.open_tree(ModelType::name())?,
            _type: Default::default(),
        })
    }
}

pub trait Model: Serialize + DeserializeOwned {
    fn name() -> &'static str;
    fn key(&self) -> String;
}

#[derive(Clone)]
pub struct Tree<ModelType: Model> {
    pub tree: sled::Tree,
    pub _type: std::marker::PhantomData<ModelType>,
}

impl<ModelType: Model> Tree<ModelType> {
    pub fn put(&self, model: &ModelType) -> eyre::Result<()> {
        let key = model.key();
        let value = bincode::serialize(model)?;
        self.tree.insert(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> eyre::Result<Option<ModelType>> {
        Ok(match self.tree.get(key)? {
            Some(bytes) => {
                let value: ModelType = bincode::deserialize(&bytes)?;
                assert_eq!(key, value.key());
                Some(value)
            }
            None => None
        })
    }

    pub async fn get_or_set<T>(&self, key: &str, setter: impl FnOnce() -> T) -> eyre::Result<ModelType> where
        T: Future<Output=ModelType>,
     {
        match self.get(key)? {
            Some(value) => Ok(value),
            None => {
                let value = setter().await;
                assert_eq!(key, value.key());
                self.put(&value)?;
                Ok(value)
            }
        }
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

fn test(k: GameSearchKey, t: Tree<GameSearch>) {
    t.get_or_set("some_key", || async {
        GameSearch {
            key: k.clone(),
            response: json!([])
        }
    });
}

impl Model for GameSearch {
    fn name() -> &'static str {
        "GameSearch"
    }

    fn key(&self) -> String {
        self.key.name_contains.clone()
    }

}
