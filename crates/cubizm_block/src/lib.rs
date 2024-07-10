use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetApp, AssetEvent, AssetServer, LoadedFolder};

use bevy::prelude::*;
use definition::Block;
use loader::BlockLoader;

use texture_atlas::BlockInfoFolder;
pub use texture_atlas::*;

use cubizm_core::AppState;

pub mod definition;
mod loader;
pub mod texture_atlas;
mod voxel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum BlockLoadingState {
    #[default]
    Pending,
    LoadBlockInfo,
    Finished,
}

fn load_blocks(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BlockInfoFolder::new(
        asset_server.load_folder("blocks/info"),
    ));
}

fn check_block(
    mut next_state: ResMut<NextState<BlockLoadingState>>,
    block_info_folder: Res<BlockInfoFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(block_info_folder.clone_handle()) {
            next_state.set(BlockLoadingState::Finished);
        }
    }
}

fn move_to_loaded_block(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::BlocksLoaded);
}

fn begin_loading_blocks(mut next_state: ResMut<NextState<BlockLoadingState>>) {
    next_state.set(BlockLoadingState::LoadBlockInfo);
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
                (texture_atlas::setup_texture_atlas, move_to_loaded_block),
            );
    }
}
