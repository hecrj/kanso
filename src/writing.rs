use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Writing {
    filepath: PathBuf,
    content: String,
    version: Version,
    last_save: Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u64);

impl Writing {
    pub async fn load(filepath: impl AsRef<Path>) -> Result<Self, Error> {
        let path = filepath.as_ref().to_path_buf();
        let exists = tokio::fs::try_exists(filepath).await?;

        let content = if exists {
            tokio::fs::read_to_string(&path).await?
        } else {
            String::new()
        };

        Ok(Self {
            filepath: path,
            content,
            version: Version(0),
            last_save: Version(0),
        })
    }

    pub fn save(&mut self) -> impl Future<Output = Result<(), Error>> {
        self.last_save = self.version;

        let filepath = self.filepath.clone();
        let content = self.content.clone();

        async move {
            tokio::fs::write(filepath, content).await?;

            Ok(())
        }
    }

    pub fn filepath(&self) -> &Path {
        &self.filepath
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn is_dirty(&self) -> bool {
        self.version != self.last_save
    }

    pub fn word_count(&self) -> usize {
        self.content.unicode_words().count()
    }

    pub fn write(&mut self, character: char) {
        self.content.push(character);
        self.version = Version(self.version.0 + 1);
    }

    pub fn amend(&mut self) {
        let _ = self.content.pop();
        self.version = Version(self.version.0 + 1);
    }
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("IO operation failed: {0}")]
    IOFailed(io::ErrorKind),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(error.kind())
    }
}
