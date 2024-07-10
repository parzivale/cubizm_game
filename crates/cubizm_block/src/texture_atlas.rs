use bevy::{asset::LoadedFolder, prelude::*, render::texture::ImageSampler};

use crate::definition::Block;

#[derive(Resource, Default)]
pub(crate) struct BlockInfoFolder(Handle<LoadedFolder>);

#[derive(Resource)]
pub struct BlockAtlas {
    image: Handle<Image>,
    texture_atlas_layout: TextureAtlasLayout,
}

impl BlockInfoFolder {
    pub(crate) fn new(handle: impl Into<Handle<LoadedFolder>>) -> Self {
        Self(handle.into())
    }

    pub(crate) fn clone_handle(&self) -> Handle<LoadedFolder> {
        Handle::clone(&self.0)
    }
}

#[allow(dead_code)]
impl BlockAtlas {
    pub(crate) fn new(
        image: impl Into<Handle<Image>>,
        texture_atlas_layout: impl Into<TextureAtlasLayout>,
    ) -> Self {
        Self {
            image: image.into(),
            texture_atlas_layout: texture_atlas_layout.into(),
        }
    }

    pub fn clone_image(&self) -> Handle<Image> {
        Handle::clone(&self.image)
    }

    pub fn get_texture_atlas_layout(&self) -> &TextureAtlasLayout {
        &self.texture_atlas_layout
    }
}

pub(crate) fn setup_texture_atlas(
    loaded_folders: Res<Assets<LoadedFolder>>,
    block_info_handles: Res<BlockInfoFolder>,
    mut textures: ResMut<Assets<Image>>,
    blocks: Res<Assets<Block>>,
    mut commands: Commands,
) {
    let loaded_folder = loaded_folders.get(&block_info_handles.0).unwrap();
    let (texture_atlas_linear, linear_texture) = create_texture_atlas(
        loaded_folder,
        None,
        Some(ImageSampler::nearest()),
        &mut textures,
        Res::clone(&blocks),
    );
    commands.insert_resource(BlockAtlas {
        texture_atlas_layout: texture_atlas_linear,
        image: linear_texture,
    });
}

pub(crate) fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
    blocks: Res<Assets<Block>>,
) -> (TextureAtlasLayout, Handle<Image>) {
    // Build a texture atlas using the individual sprites
    let mut texture_atlas_builder =
        TextureAtlasBuilder::default().padding(padding.unwrap_or_default());
    for handle in folder.handles.iter() {
        let block_id = handle.id().typed_unchecked::<Block>();
        let Some(block) = blocks.get(block_id) else {
            warn!(
                "{:?} did not resolve to an `Block` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        if let Block::TileEntity(_) = &block {
            continue;
        }

        let Some(texture_handle) = block.voxel_texture() else {
            continue;
        };

        let id = texture_handle.id();

        let Some(texture) = textures.get(id) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                texture_handle.path().unwrap()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture) = texture_atlas_builder.finish().unwrap();
    let texture = textures.add(texture);
    // Update the sampling settings of the texture atlas
    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (texture_atlas_layout, texture)
}
