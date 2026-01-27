//! In-memory representation of a ZIP archive.

use {
    crate::{
        generic::default, never, output_buffer, panic, zip::write_zip::write_zip, OutputBuffer,
        ToZip, ZipConfiguration, ZipEntry, ZipEntryComparison,
    },
    serde::{Deserialize, Serialize},
    std::{io::Read, path::Path},
    tracing::{debug, instrument},
};

/// In-memory representation of a ZIP file's essential archive contents.
#[derive(Debug, Clone, Default)]
pub struct Zip {
    pub configuration: ZipConfiguration,
    pub files: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Zip {
    /// Creates a new [`Zip`] from the given data.
    pub fn new(data: &impl ToZip) -> Self {
        data.to_zip().into_owned()
    }

    pub fn new_with_files(files: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self { files, ..default() }
    }

    #[instrument(skip_all)]
    /// Creates a new [`Zip`] from the file or directory at the given path.
    pub fn new_from_path(path: impl AsRef<Path>) -> Result<Self, panic> {
        let path = path.as_ref();
        let mut files = Vec::<(Vec<u8>, Vec<u8>)>::new();

        let walkdir = walkdir::WalkDir::new(path);
        for entry in walkdir {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            // XXX: zip requires `/`-delimited paths, and if we're creating from
            // the filesystem directly I guess we're responsible for that,
            // although we don't want to touch users' own input if they're
            // manually specifying paths.
            let adding = entry.path().to_path_buf();
            debug!("adding {adding:?}");
            let contents = std::fs::read(adding)?;
            files.push((path.to_str().unwrap().as_bytes().to_vec(), contents));
        }

        Ok(Self::new_with_files(files))
    }

    pub fn sort_by(&mut self, comparison: ZipEntryComparison) {
        self.files.sort_by(|a, b| {
            comparison(
                &ZipEntry {
                    name: &a.0,
                    body: &a.1,
                },
                &ZipEntry {
                    name: &b.0,
                    body: &b.1,
                },
            )
        })
    }

    #[instrument(skip_all)]
    /// Serializes this [`Png`] as a PNG image file.
    pub fn serialize(&self) -> OutputBuffer {
        let mut buffer = output_buffer();

        write_zip(
            &mut buffer,
            self.files
                .iter()
                .map(|(a, b)| (a.as_slice(), b.as_slice()))
                .collect::<Vec<_>>()
                .as_slice(),
            &[],
        )
        .unwrap();
        buffer
    }

    /// Deserializes a ZIP archive file into a [`Zip`].
    pub fn read(_input: &impl Read) -> Result<Self, panic> {
        unimplemented!()
    }

    /// Deserialize a ZIP archive file into a [`Zip`] from a byte vector.
    pub fn read_slice(input: &[u8]) -> Result<Self, never> {
        Ok(Self::read(&input)?)
    }
}

impl Serialize for Zip {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_bytes(self.serialize().as_ref())
    }
}

impl<'de> Deserialize<'de> for Zip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let bytes: &[u8] = serde_bytes::deserialize(deserializer)?;
        Self::read_slice(bytes).map_err(serde::de::Error::custom)
    }
}
