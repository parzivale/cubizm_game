use std::fmt::Debug;

use bevy::asset::{Handle, LoadedFolder};
use bevy::prelude::*;

use crate::chunk::Chunk;
use cubizm_block::definition::Block;
use cubizm_block::texture_atlas::BlockAtlas;

use cubizm_core::AppState;

use definition::*;

mod definition;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum ChunkLoadingState {
    #[default]
    Pending,
    LoadChunks,
    Finished,
}

#[derive(Resource, Default)]
pub struct ChunksFolder(Handle<LoadedFolder>);

fn load_chunks(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ChunksFolder(asset_server.load_folder("world/chunks")));
}

fn check_chunk(
    mut next_state: ResMut<NextState<ChunkLoadingState>>,
    block_info_folder: Res<ChunksFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&block_info_folder.0) {
            next_state.set(ChunkLoadingState::Finished);
        }
    }
}

fn create_chunk_resource(
    mut commands: Commands,
    loaded_folders: Res<Assets<LoadedFolder>>,
    chunk_handles: Res<ChunksFolder>,
    mut assets_chunks: ResMut<Assets<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_atlas: Res<BlockAtlas>,
    blocks: Res<Assets<Block>>,
) {
    let mut chunks = Chunks::new();
    let loaded_folder = loaded_folders.get(&chunk_handles.0).unwrap();
    for handle in loaded_folder.handles.iter() {
        let chunk_id = handle.id().typed_unchecked::<Chunk>();
        let Some(chunk) = assets_chunks.get(chunk_id) else {
            warn!(
                "{:?} did not resolve to an `Chunk` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        chunks.insert_chunk_and_regenerate(
            chunk.to_owned(),
            chunk.position,
            &mut commands,
            &mut meshes,
            &mut materials,
            Res::clone(&texture_atlas),
            &mut assets_chunks,
            Res::clone(&blocks),
        );
    }
    commands.insert_resource(chunks);
}

fn move_to_loaded_chunks(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::ChunksLoaded);
}

fn begin_loading_chunks(mut next_state: ResMut<NextState<ChunkLoadingState>>) {
    next_state.set(ChunkLoadingState::LoadChunks);
}

pub struct ChunksPlugin;
impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ChunkLoadingState>()
            .init_asset::<Chunk>()
            .init_asset_loader::<crate::chunk::ChunkLoader>()
            .add_systems(OnEnter(AppState::BlocksLoaded), begin_loading_chunks)
            .add_systems(OnEnter(ChunkLoadingState::LoadChunks), load_chunks)
            .add_systems(
                Update,
                check_chunk.run_if(in_state(ChunkLoadingState::LoadChunks)),
            )
            .add_systems(
                OnEnter(ChunkLoadingState::Finished),
                (create_chunk_resource, move_to_loaded_chunks),
            );
    }
}
