use bevy::asset::ron;

use block_mesh::VoxelVisibility::Opaque;
use cubizm::SerializedBlock;

fn main() {
    let mut block = SerializedBlock {
        mesh: None,
        name: "Dirt".to_string(),
        texture: Some("blocks/textures/dirt.jpg".to_string()),
        visibility: Opaque,
    };
    std::fs::write(
        "./assets/blocks/info/dirt.block",
        ron::ser::to_string(&block).unwrap(),
    )
    .unwrap();
}
