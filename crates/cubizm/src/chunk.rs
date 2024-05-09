use bevy::asset::{AssetLoader, AsyncReadExt, BoxedFuture, LoadContext, ron};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use thiserror::Error;

use block_mesh::{RIGHT_HANDED_Y_UP_CONFIG, UnitQuadBuffer, visible_block_faces};
use block_mesh::ndshape::{ConstShape, ConstShape3u32};

use crate::block::Block;

pub const CHUNK_SIZE: u32 = 16;
pub type ChunkShape = ConstShape3u32<{ CHUNK_SIZE + 2 }, { CHUNK_SIZE + 2 }, { CHUNK_SIZE + 2 }>;

#[derive(Clone, Copy)]
pub enum ChunkFace {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Serialize, Deserialize)]
struct SerializedChunk {
    #[serde(with = "BigArray")]
    pub blocks: [String; ChunkShape::SIZE as usize],
    pub position: IVec3,
}

/// Internal representation of a chunk. This does not contain the final [Mesh],
/// see [ChunkEntity] instead if a mesh is needed
#[derive(Asset, TypePath, Clone, Debug)]
pub struct Chunk {
    pub blocks: [Block; ChunkShape::SIZE as usize],
    pub position: IVec3,
}

impl Chunk {
    pub fn get_own_face_indicies(
        face: ChunkFace,
    ) -> [u32; { (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) } as usize] {
        let mut indicies = [0; { (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) } as usize];
        let mut index = 0;
        match face {
            ChunkFace::Front => {
                for x in 0..CHUNK_SIZE + 2 {
                    for y in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, y, 1]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Back => {
                for x in 0..CHUNK_SIZE + 2 {
                    for y in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, y, CHUNK_SIZE]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Top => {
                for x in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, CHUNK_SIZE, z]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Bottom => {
                for x in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, 1, z]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Right => {
                for y in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([CHUNK_SIZE, y, z]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Left => {
                for y in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([1, y, z]);
                        index += 1;
                    }
                }
            }
        };
        indicies
    }

    pub fn get_other_face_indicies(
        face: ChunkFace,
    ) -> [u32; { (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) } as usize] {
        let mut indicies = [0; { (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) } as usize];
        let mut index = 0;
        match face {
            ChunkFace::Front => {
                for x in 0..CHUNK_SIZE + 2 {
                    for y in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, y, 0]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Back => {
                for x in 0..CHUNK_SIZE + 2 {
                    for y in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, y, CHUNK_SIZE + 1]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Top => {
                for x in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, CHUNK_SIZE + 1, z]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Bottom => {
                for x in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([x, 0, z]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Right => {
                for y in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([CHUNK_SIZE + 1, y, z]);
                        index += 1;
                    }
                }
            }
            ChunkFace::Left => {
                for y in 0..CHUNK_SIZE + 2 {
                    for z in 0..CHUNK_SIZE + 2 {
                        indicies[index] = ChunkShape::linearize([0, y, z]);
                        index += 1;
                    }
                }
            }
        };
        indicies
    }

    pub fn gen_geometry(&self, texture_atlas: &TextureAtlasLayout) -> Mesh {
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

        let mut buffer = UnitQuadBuffer::new();
        visible_block_faces(
            &self.blocks,
            &ChunkShape {},
            [0; 3],
            [CHUNK_SIZE + 1; 3],
            &RIGHT_HANDED_Y_UP_CONFIG.faces,
            &mut buffer,
        );
        let num_indices = buffer.num_quads() * 6;
        let num_vertices = buffer.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut tex_coords = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                normals.extend_from_slice(&face.quad_mesh_normals());
                let texture = &quad
                    .voxel
                    .texture
                    .clone()
                    .expect("The voxel has no associated texture");
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));

                let index = texture_atlas
                    .get_texture_index(texture)
                    .expect("image hasn't been loaded into texture atlas");

                let rect = texture_atlas.textures[index];
                let width = rect.width() / texture_atlas.size[0];
                let height = rect.height() / texture_atlas.size[1];
                let start_pos: [f32; 2] = (rect.min / texture_atlas.size).into();

                fn calculate_face_uv(
                    face_no: f32,
                    width: f32,
                    height: f32,
                    start_pos: [f32; 2],
                ) -> [[f32; 2]; 4] {
                    let (start_pos_x, start_pos_y) = start_pos.into();
                    let base_face: [[f32; 2]; 4] =
                        [[1., 0.], [0., 0.], [1., -1. / 6.], [0., -1. / 6.]];

                    base_face
                        .map(|mut x| {
                            x[1] += face_no * 1. / 6.;
                            x
                        })
                        .map(|xy| [xy[0] * width, xy[1] * height])
                        .map(|xy| [xy[0] + start_pos_x, xy[1] + start_pos_y])
                }

                let face_tex_coords: [[f32; 2]; 4] = match face.signed_normal().into() {
                    (1, 0, 0) => calculate_face_uv(1., width, height, start_pos),
                    (0, 1, 0) => calculate_face_uv(2., width, height, start_pos),
                    (0, 0, 1) => calculate_face_uv(3., width, height, start_pos),
                    (-1, 0, 0) => calculate_face_uv(4., width, height, start_pos),
                    (0, -1, 0) => calculate_face_uv(5., width, height, start_pos),
                    (0, 0, -1) => calculate_face_uv(6., width, height, start_pos),
                    _ => calculate_face_uv(1., width, height, start_pos),
                };
                tex_coords.extend_from_slice(&face_tex_coords);
            }
        }
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(tex_coords),
        )
        .with_inserted_indices(Indices::U32(indices.clone()))
    }
}

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
            let mut blocks = <[Block; ChunkShape::SIZE as usize]>::try_from(vec![
                Block::default();
                ChunkShape::SIZE
                    as usize
            ])
            .unwrap();

            for (index, block) in ron.blocks.iter().enumerate() {
                let loaded = load_context.load_direct(block).await?;
                blocks[index] = loaded.get::<Block>().unwrap().clone();
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
