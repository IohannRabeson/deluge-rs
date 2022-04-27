use std::{path::{Path, PathBuf}, collections::HashMap, rc::Rc};

use bwavfile::{WaveReader, WaveFmt, Cue};

use crate::SamplePosition;

#[derive(Clone)]
pub struct WavMetadata {
    pub frame_count: SamplePosition,
    pub format: WaveFmt, 
    pub cue_points: Rc<Vec<Cue>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to find file '{0}'")]
    FileNotFound(PathBuf),
    #[error("Unable to read metadata: {0}")]
    ParserError(#[from] bwavfile::Error),
}

pub trait WavMetadataProvider {
    fn read_metadata(&self, path: &Path) -> Result<WavMetadata, Error>;
}

#[derive(Default)]
pub struct LocalWavMetadataProvider;

impl WavMetadataProvider for LocalWavMetadataProvider {
    fn read_metadata(&self, path: &Path) -> Result<WavMetadata, Error> {
        let mut reader = WaveReader::open(path)?;

        Ok(WavMetadata{
            frame_count: reader.frame_length()?.into(),
            format: reader.format()?,
            cue_points: Rc::new(reader.cue_points()?),
        })
    }
}

#[derive(Default)]
pub struct InMemoryWavMetadataProvider<'l> {
    pub metadata: HashMap<&'l Path, WavMetadata>,
}

impl<'l> InMemoryWavMetadataProvider<'l> {
    pub fn new(metadata: HashMap<&'l Path, WavMetadata>) -> Self {
        Self {
            metadata
        }
    }
}

impl<'l> WavMetadataProvider for InMemoryWavMetadataProvider<'l> {
    fn read_metadata(&self, path: &Path) -> Result<WavMetadata, Error> {
        match self.metadata.get(path) {
            Some(metadata) => Ok(metadata.clone()),
            None => Err(Error::FileNotFound(path.to_path_buf())),
        }
    }
}