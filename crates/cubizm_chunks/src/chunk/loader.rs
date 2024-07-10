use bevy::{
    asset::{io::Reader, ron, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use block_mesh::ndshape::ConstShape;
use thiserror::Error;

use crate::{Chunk, ChunkShape, SerializedChunk};
use cubizm_block::definition::Block;

#[derive(Debug, Error)]
pub enum ChunkLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error(transparent)]
    LoadDirectError(#[from] bevy::asset::LoadDirectError),
}

#[derive(Default)]
pub struct ChunkLoader;

impl AssetLoader for ChunkLoader {
    type Asset = Chunk;
    type Settings = ();
    type Error = ChunkLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let ron: SerializedChunk = ron::de::from_bytes(&bytes)?;
            let mut blocks: Vec<Handle<Block>> = Vec::with_capacity(ChunkShape::SIZE as usize);

            for block in ron.blocks.iter() {
                let loaded = load_context.load(block);

                blocks.push(loaded);
            }
            Ok(Chunk {
                blocks,
                position: ron.position,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["chunk"]
    }
}
