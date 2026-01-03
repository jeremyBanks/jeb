#![allow(unused)]
//! Serializes and deserializes snapshots of Git repositories for use in tests.
//! This does not support everything and is not suitable for production use, but
//! it _should_ throw an error instead of incorrectly round-tripping anything.
//!
//! Notably, we do not support annotated tags.

use std::{collections::BTreeMap, sync::Arc};

struct RepositorySnapshot {
    head: Option<String>,
    index: Tree,
    refs: BTreeMap<String, Commit>,
}

struct Object {
    id: ObjectId,
    value: ObjectArc,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
struct ObjectId([u8; 20]);


enum ObjectArc {
    Blob(Arc<Blob>),
    Commit(Arc<Commit>),
    Tree(Arc<Tree>),
}


struct Blob {
    id: ObjectId,
    data: Vec<u8>,
}


struct Commit {
    parents: Vec<Commit>,
    message: String,
    tree: Tree,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
struct Tree {
    entries: BTreeMap<String, TreeEntry>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
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