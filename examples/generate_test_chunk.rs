use bevy::asset::ron;
use bevy::math::IVec3;

use block_mesh::ndshape::ConstShape;
use cubizm_chunks::{ChunkShape, SerializedChunk, CHUNK_SIZE};

fn main() {
    let mut chunk = SerializedChunk::default();
    for x in 1..CHUNK_SIZE + 1 {
        for y in 1..CHUNK_SIZE + 1 {
            for z in 1..CHUNK_SIZE + 1 {
                chunk.blocks[ChunkShape::linearize([x, y, z]) as usize] =
                    "blocks/info/dirt.block".to_string();
            }
        }
    }
    chunk.position = IVec3::new(1, 0, 0);
    std::fs::write(
        "./assets/world/chunks/test.chunk",
        ron::ser::to_string(&chunk).unwrap(),
    )
    .unwrap();
}
