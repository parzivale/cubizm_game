use bevy::app::{App, Plugin, Update};
use bevy::asset::{
    Asset, AssetApp, AssetEvent, AssetLoader, Assets, AssetServer, AsyncReadExt, BoxedFuture, Handle,
    LoadContext, LoadedFolder, ron,
};
use bevy::asset::io::Reader;
use bevy::log::warn;
use bevy::math::UVec2;
use bevy::prelude::{
    Commands, EventReader, Image, in_state, IntoSystemConfigs, Mesh, NextState, OnEnter, Res,
    ResMut, Resource, States, TextureAtlasBuilder, TextureAtlasLayout, TypePath,
};
use bevy::render::texture::ImageSampler;
use bevy::utils::thiserror::Error;
use serde::{Deserialize, Serialize};

use block_mesh::{Voxel, VoxelVisibility};

use crate::AppState;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum BlockLoadingState {
    #[default]
    Pending,
    LoadBlockInfo,
    Finished,
}

#[derive(Resource, Default)]
struct BlockInfoFolder(Handle<LoadedFolder>);

#[derive(Resource)]
pub struct BlockAtlas {
    pub image: Handle<Image>,
    pub texture_atlas_layout: TextureAtlasLayout,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedBlock {
    mesh: Option<String>,
    name: String,
    texture: Option<String>,
    visibility: VoxelVisibility,
}

#[derive(Clone, Debug, Asset, TypePath)]
pub struct Block {
    pub mesh: Option<Handle<Mesh>>,
    pub name: String,
    pub texture: Option<Handle<Image>>,
    pub visibility: VoxelVisibility,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            mesh: None,
            name: "".into(),
            texture: None,
            visibility: VoxelVisibility::Empty,
        }
    }
}

impl Block {
    pub fn air() -> Self {
        Block {
            mesh: None,
            name: "Air".into(),
            texture: None,
            visibility: VoxelVisibility::Empty,
        }
    }
}

impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        self.visibility
    }
}

impl Voxel for &Block {
    fn get_visibility(&self) -> VoxelVisibility {
        self.visibility
    }
}

impl Voxel for Box<Block> {
    fn get_visibility(&self) -> VoxelVisibility {
        self.visibility
    }
}

#[derive(Default)]
struct BlockLoader;

#[derive(Debug, Error)]
enum BlockLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error(transparent)]
    LoadDirectError(#[from] bevy::asset::LoadDirectError),
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
            let mesh = ron.mesh.map(|path| load_context.load(path));

            let texture = ron.texture.map(|path| load_context.load(path));

            Ok(Block {
                mesh,
                texture,
                name: ron.name,
                visibility: ron.visibility,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["block"]
    }
}

fn setup_texture_atlas(
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

fn create_texture_atlas(
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

        let Some(texture_handle) = &block.texture else {
            continue;
        };

        let Some(texture) = textures.get(texture_handle) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                texture_handle.path().unwrap()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(texture_handle.into()), texture);
    }

    let (texture_atlas_layout, texture) = texture_atlas_builder.finish().unwrap();
    let texture = textures.add(texture);
    // Update the sampling settings of the texture atlas
    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (texture_atlas_layout, texture)
}

fn load_blocks(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BlockInfoFolder(asset_server.load_folder("blocks/info")));
}

fn check_block(
    mut next_state: ResMut<NextState<BlockLoadingState>>,
    block_info_folder: Res<BlockInfoFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&block_info_folder.0) {
            next_state.set(BlockLoadingState::Finished);
        }
    }
}

fn move_to_loaded_block(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::BlocksLoaded);
}

fn begin_loading_blocks(mut next_state: ResMut<NextState<BlockLoadingState>>) {
    next_state.set(crate::block::BlockLoadingState::LoadBlockInfo);
}

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Block>()
            .init_asset_loader::<BlockLoader>()
            .init_state::<BlockLoadingState>()
            .add_systems(OnEnter(AppState::Setup), begin_loading_blocks)
            .add_systems(OnEnter(BlockLoadingState::LoadBlockInfo), load_blocks)
            .add_systems(
                Update,
                check_block.run_if(in_state(BlockLoadingState::LoadBlockInfo)),
            )
            .add_systems(
                OnEnter(BlockLoadingState::Finished),
                (setup_texture_atlas, move_to_loaded_block),
            );
    }
}
