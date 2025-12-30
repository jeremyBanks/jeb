#![allow(unused)]
//! Serializes and deserializes snapshots of Git repositories for use in tests.
//! This does not support everything and is not suitable for production use, but
//! it _should_ throw an error instead of incorrectly round-tripping anything.
//!
//! Notably, we do not support annotated tags.

use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

struct RepositorySnapshot {

}

struct Object {
    
}

#[derive(Serialize, Deserialize)]
struct ObjectId([u8; 20]);

#[derive(Serialize, Deserialize)]
enum ObjectArc {
    Blob(Arc<Blob>),
    Commit(Arc<Commit>),
    Tree(Arc<Tree>),
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct Blob {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Commit {
    parents: Vec<Commit>,
    message: String,
    tree: Tree,
}

#[derive(Serialize, Deserialize)]
struct Tree {
    entries: BTreeMap<String, TreeEntry>,
}

#[derive(Serialize, Deserialize)]
enum TreeEntry {
    Blob(ObjectId),
    Tree(Tree),
}

impl RepositorySnapshot {
    pub fn from_git2(repo: &git2::Repository) -> Result<Self, git2::Error> {
        todo!()
    }

    pub fn write_to_git2(&self, repo: &git2::Repository) -> Result<(), git2::Error> {
        assert!(repo.is_empty().unwrap(), "git2::Repository must be empty to write snapshot");

        todo!()
    }
}