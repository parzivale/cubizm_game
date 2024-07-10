use bevy::{
    asset::{io::Reader, ron, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use thiserror::Error;

use crate::definition::{BlockBuilderError, TileEntityBlockBuilder, VoxelBlockBuilder};

use super::definition::{Block, SerializedBlock};

#[derive(Default)]
pub struct BlockLoader;

#[derive(Debug, Error)]
pub enum BlockLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error(transparent)]
    LoadDirectError(#[from] bevy::asset::LoadDirectError),
    #[error(transparent)]
    BlockBuilderError(#[from] BlockBuilderError),
}

impl AssetLoader for BlockLoader {
    type Asset = Block;
    type Settings = ();
    type Error = BlockLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let ron: SerializedBlock = ron::de::from_bytes(&bytes)?;
            match ron {
                SerializedBlock::SerializedTileEntity(tile_entity) => {
                    let mesh = tile_entity.mesh.map(|path| load_context.load(path));

                    let texture = tile_entity.texture.map(|path| load_context.load(path));

                    let mut block = TileEntityBlockBuilder::new();
                    block.name(&tile_entity.name);

                    if let Some(mesh) = mesh {
                        block.mesh(mesh);
                    }

                    if let Some(texture) = texture {
                        block.texture(texture);
                    }

                    Ok(block.finish()?)
                }
                SerializedBlock::SerializedVoxel(voxel) => {
                    let texture = voxel.texture.map(|path| load_context.load::<Image>(path));

                    let mut block = VoxelBlockBuilder::new();
                    block.name(&voxel.name);
                    block.visibility(voxel.visibility)?;
                    if let Some(texture) = texture {
                        block.texture(texture);
                    }

                    Ok(block.finish()?)
                }
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["block"]
    }
}
