use bevy::{
    asset::{AssetPath, LoadContext},
    prelude::*,
};
use block_mesh::VoxelVisibility;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Asset, TypePath)]
pub struct VoxelBlock {
    name: String,
    texture: Option<Handle<Image>>,
    visibility: VoxelVisibility,
}

#[derive(Clone, Debug, Asset, TypePath)]
pub struct TileEntityBlock {
    mesh: Handle<Mesh>,
    name: String,
    texture: Handle<Image>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedVoxelBlock {
    pub name: String,
    pub texture: Option<String>,
    pub visibility: VoxelVisibility,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedTileEntityBlock {
    pub mesh: Option<String>,
    pub name: String,
    pub texture: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SerializedBlock {
    SerializedVoxel(SerializedVoxelBlock),
    SerializedTileEntity(SerializedTileEntityBlock),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Asset, TypePath)]
pub enum Block {
    Voxel(VoxelBlock),
    TileEntity(TileEntityBlock),
}

#[derive(Default)]
pub(crate) struct VoxelBlockBuilder {
    name: Option<String>,
    texture: Option<Handle<Image>>,
    visibility: Option<VoxelVisibility>,
}

#[derive(Default)]
pub(crate) struct TileEntityBlockBuilder {
    mesh: Option<Handle<Mesh>>,
    name: Option<String>,
    texture: Option<Handle<Image>>,
}

#[derive(Error, Debug)]
pub(crate) enum BlockBuilderError {
    #[error("Voxel name must be set")]
    UnsetNameForVoxel,
    #[error("TileEntity name must be set")]
    UnsetNameForTileEntity,
    #[error("VoxelVisibility must be set")]
    UnsetVisbility,
    #[error("Texture must be set to set VoxelVisbility to non None value")]
    UnsetTextureForVoxel,
    #[error("Texture must be set on a TileEntity")]
    UnsetTextureForTileEntity,
    #[error("Model must be set on a TileEntity")]
    UnsetModelForTileEntity,
    #[error("Texture cannot be set if VoxelVisibility is None")]
    VisbilityNoneTexture,
}

#[allow(dead_code)]
impl Block {
    pub(crate) fn air() -> Self {
        Self::Voxel(VoxelBlock {
            name: "Air".into(),
            texture: None,
            visibility: VoxelVisibility::Empty,
        })
    }

    pub fn is_voxel(&self) -> bool {
        matches!(self, Self::Voxel(_))
    }

    pub fn voxel_texture(&self) -> Option<Handle<Image>> {
        match self {
            Self::Voxel(block) => block.texture.clone(),
            _ => None,
        }
    }

    pub(crate) fn tile_entity_texture(&self) -> Option<Handle<Image>> {
        match self {
            Self::TileEntity(block) => Some(block.texture.clone()),
            _ => None,
        }
    }

    pub(crate) fn tile_entity_mesh(&self) -> Option<Handle<Mesh>> {
        match self {
            Self::TileEntity(block) => Some(block.mesh.clone()),
            _ => None,
        }
    }

    pub(crate) fn get_voxel_visibility(&self) -> VoxelVisibility {
        match self {
            Self::Voxel(block) => block.visibility,
            _ => VoxelVisibility::Empty,
        }
    }

    pub(crate) fn get_name(&self) -> &str {
        match self {
            Self::TileEntity(block) => &block.name,
            Self::Voxel(block) => &block.name,
        }
    }

    pub(crate) fn get_mesh(&self) -> Option<Handle<Mesh>> {
        match self {
            Self::TileEntity(block) => Some(block.mesh.clone()),
            _ => None,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::air()
    }
}

impl VoxelBlockBuilder {
    pub(crate) fn new() -> Self {
        Self { ..default() }
    }

    pub(crate) fn visibility(
        &mut self,
        visibility: impl Into<VoxelVisibility>,
    ) -> Result<&mut Self, BlockBuilderError> {
        let visibility = visibility.into();
        match &visibility {
            VoxelVisibility::Empty if self.texture.is_some() => {
                return Err(BlockBuilderError::UnsetTextureForVoxel)
            }
            _ => self.visibility = Some(visibility),
        };

        Ok(self)
    }

    pub(crate) fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }
    pub(crate) fn texture(&mut self, texture: impl Into<Handle<Image>> + Clone) -> &mut Self {
        self.texture = Some(texture.into());
        self
    }

    pub(crate) fn finish(self) -> Result<Block, BlockBuilderError> {
        let Some(name) = self.name else {
            return Err(BlockBuilderError::UnsetNameForVoxel);
        };
        let Some(visibility) = self.visibility else {
            return Err(BlockBuilderError::UnsetVisbility);
        };

        Ok(Block::Voxel(VoxelBlock {
            name,
            texture: self.texture,
            visibility,
        }))
    }
}

impl TileEntityBlockBuilder {
    pub(crate) fn new() -> Self {
        Self { ..default() }
    }

    pub(crate) fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub(crate) fn mesh(&mut self, mesh: impl Into<Handle<Mesh>>) -> &mut Self {
        self.mesh = Some(mesh.into());
        self
    }

    pub(crate) fn texture(&mut self, texture: impl Into<Handle<Image>>) -> &mut Self {
        self.texture = Some(texture.into());
        self
    }

    pub(crate) fn finish(self) -> Result<Block, BlockBuilderError> {
        let Some(name) = self.name else {
            return Err(BlockBuilderError::UnsetNameForTileEntity);
        };

        let Some(texture) = self.texture else {
            return Err(BlockBuilderError::UnsetTextureForTileEntity);
        };

        let Some(mesh) = self.mesh else {
            return Err(BlockBuilderError::UnsetModelForTileEntity);
        };

        Ok(Block::TileEntity(TileEntityBlock {
            name,
            texture,
            mesh,
        }))
    }
}
