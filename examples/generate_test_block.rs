use bevy::asset::ron;

use block_mesh::VoxelVisibility::Opaque;
use cubizm_block::definition::{SerializedBlock, SerializedVoxelBlock};

fn main() {
    let block = SerializedBlock::SerializedVoxel(SerializedVoxelBlock {
        name: "Test".to_string(),
        texture: Some("blocks/textures/test.jpg".to_string()),
        visibility: Opaque,
    });
    std::fs::write(
        "./assets/blocks/info/test.block",
        ron::ser::to_string(&block).unwrap(),
    )
    .unwrap();
}
