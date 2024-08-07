use crate::ChunkShape;
use crate::Opposite;
use crate::{Chunk, ChunkFace};
use bevy::{prelude::*, utils::HashMap};
use block_mesh::ndshape::ConstShape;
use cubizm_block::{definition::Block, texture_atlas::BlockAtlas};
use std::ops::Add;
use thiserror::Error;

/// The chunk representation of the world
#[derive(Resource, Default)]
pub struct Chunks {
    pub chunks: HashMap<IVec3, ChunkEntity>,
}

/// Stores the [Chunk] data and its [Mesh], use the [Chunks] resource to access.
#[derive(Debug)]
pub struct ChunkEntity {
    #[allow(unused)]
    entity: Entity,
    pub chunk: Handle<Chunk>,
    pub mesh_handle: Handle<Mesh>,
}

impl From<&mut ChunkEntity> for AssetId<Chunk> {
    fn from(value: &mut ChunkEntity) -> Self {
        value.chunk.clone().id()
    }
}

impl From<ChunkEntity> for AssetId<Chunk> {
    fn from(value: ChunkEntity) -> Self {
        value.chunk.id()
    }
}

impl From<&mut ChunkEntity> for Handle<Chunk> {
    fn from(value: &mut ChunkEntity) -> Self {
        value.chunk.clone()
    }
}
impl From<ChunkEntity> for Handle<Chunk> {
    fn from(value: ChunkEntity) -> Self {
        value.chunk
    }
}

#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("Chunk could not be found")]
    ChunkNotFound,
}

impl Chunks {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Grabs the neighbouring chunk by a given [direction](ChunkFace)
    fn get_neighbouring_chunk_mut(
        &mut self,
        position: IVec3,
        neighbour: ChunkFace,
    ) -> Option<&mut ChunkEntity> {
        // front position = negative z
        let front_position = position.add(IVec3::from_array([0, 0, -1]));
        // back position = positive z
        let back_position = position.add(IVec3::from_array([0, 0, 1]));
        // top position = positive y
        let top_position = position.add(IVec3::from_array([0, 1, 0]));
        // bottom position = negative y
        let bottom_position = position.add(IVec3::from_array([0, -1, 0]));
        // right position = positive x
        let right_position = position.add(IVec3::from_array([1, 0, 0]));
        // left position = negative x
        let left_position = position.add(IVec3::from_array([-1, 0, 0]));
        match neighbour {
            ChunkFace::Top => self.chunks.get_mut(&top_position),
            ChunkFace::Bottom => self.chunks.get_mut(&bottom_position),
            ChunkFace::Front => self.chunks.get_mut(&front_position),
            ChunkFace::Back => self.chunks.get_mut(&back_position),
            ChunkFace::Right => self.chunks.get_mut(&right_position),
            ChunkFace::Left => self.chunks.get_mut(&left_position),
        }
    }

    /// Inserts a [Chunk] at a given [position](IVec3), does NOT update neighbours
    /// use [insert_chunk_and_regenerate](Chunks::insert_chunk_and_regenerate) to update neighbours on insertion or
    /// manually call [regenerate_chunk_at](Chunks::regenerate_chunk_at) to update neighbours
    #[allow(clippy::too_many_arguments)]
    pub fn insert_chunk(
        &mut self,
        chunk: Chunk,
        position: IVec3,
        texture_atlas: Res<BlockAtlas>,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        chunks: &mut ResMut<Assets<Chunk>>,
        blocks: Res<Assets<Block>>,
    ) {
        let mesh = chunk.gen_geometry(texture_atlas.get_texture_atlas_layout(), blocks);
        let chunk_handle = chunks.add(chunk);
        let mesh_handle = meshes.add(mesh);

        let entity = commands
            .spawn(PbrBundle {
                transform: Transform::from_xyz(
                    position.x as f32 * 16.,
                    position.y as f32 * 16.,
                    position.z as f32 * 16.,
                ),
                mesh: mesh_handle.clone(),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(texture_atlas.clone_image()),
                    ..default()
                }),
                ..default()
            })
            .id();
        let chunk_entity = ChunkEntity {
            entity,
            chunk: chunk_handle,
            mesh_handle,
        };

        self.chunks.insert(position, chunk_entity);
    }

    /// Regenerate a chunk and its neighbours
    pub fn regenerate_chunk_at(
        &mut self,
        position: IVec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        texture_atlas_layout: &TextureAtlasLayout,
        chunks: &mut ResMut<Assets<Chunk>>,
        blocks: Res<Assets<Block>>,
    ) -> Result<(), ChunkError> {
        let own_entity = self
            .chunks
            .get_mut(&position)
            .ok_or(ChunkError::ChunkNotFound)?;

        let own_handle = &mut own_entity.chunk.clone();
        let mut own = chunks.get(own_handle.to_owned()).unwrap().to_owned();
        let handle = own_entity.mesh_handle.clone();
        fn create_and_update_geometry(
            other_chunk: &mut Chunk,
            chunk: &mut Chunk,
            meshes: &mut ResMut<Assets<Mesh>>,
            texture_atlas_layout: &TextureAtlasLayout,
            chunk_face: ChunkFace,
            mesh_handle: Handle<Mesh>,
            blocks: Res<Assets<Block>>,
        ) {
            let chunk_own_indicies = Chunk::get_own_face_indicies(chunk_face);
            let chunk_other_indicies = Chunk::get_other_face_indicies(chunk_face);
            let other_chunk_own_indicies = Chunk::get_own_face_indicies(chunk_face.opposite());
            let other_chunk_other_indicies = Chunk::get_other_face_indicies(chunk_face.opposite());
            for ((chunk_own, chunk_other), (front_own, front_other)) in
                chunk_own_indicies.iter().zip(chunk_other_indicies).zip(
                    other_chunk_own_indicies
                        .iter()
                        .zip(other_chunk_other_indicies),
                )
            {
                chunk.blocks[chunk_other as usize] =
                    other_chunk.blocks[*front_own as usize].clone();
                other_chunk.blocks[front_other as usize] =
                    chunk.blocks[*chunk_own as usize].clone();
                let other_chunk_geometry =
                    other_chunk.gen_geometry(texture_atlas_layout, Res::clone(&blocks));
                meshes.insert(mesh_handle.clone(), other_chunk_geometry);
            }
        }

        if let Some(front) = self.get_neighbouring_chunk_mut(position, ChunkFace::Front) {
            let mesh_handle = front.mesh_handle.clone();
            let front = chunks.get_mut(front.chunk.clone()).unwrap();
            create_and_update_geometry(
                front,
                &mut own,
                meshes,
                texture_atlas_layout,
                ChunkFace::Front,
                mesh_handle,
                Res::clone(&blocks),
            );
        }
        if let Some(back) = self.get_neighbouring_chunk_mut(position, ChunkFace::Back) {
            let mesh_handle = back.mesh_handle.clone();
            let back = chunks.get_mut(back.chunk.clone()).unwrap();
            create_and_update_geometry(
                back,
                &mut own,
                meshes,
                texture_atlas_layout,
                ChunkFace::Back,
                mesh_handle,
                Res::clone(&blocks),
            );
        }
        if let Some(top) = self.get_neighbouring_chunk_mut(position, ChunkFace::Top) {
            let mesh_handle = top.mesh_handle.clone();
            let top = chunks.get_mut(top.chunk.clone()).unwrap();
            create_and_update_geometry(
                top,
                &mut own,
                meshes,
                texture_atlas_layout,
                ChunkFace::Top,
                mesh_handle,
                Res::clone(&blocks),
            );
        }
        if let Some(bottom) = self.get_neighbouring_chunk_mut(position, ChunkFace::Bottom) {
            let mesh_handle = bottom.mesh_handle.clone();
            let bottom = chunks.get_mut(bottom.chunk.clone()).unwrap();
            create_and_update_geometry(
                bottom,
                &mut own,
                meshes,
                texture_atlas_layout,
                ChunkFace::Bottom,
                mesh_handle,
                Res::clone(&blocks),
            );
        }
        if let Some(right) = self.get_neighbouring_chunk_mut(position, ChunkFace::Right) {
            let mesh_handle = right.mesh_handle.clone();
            let right = chunks.get_mut(right.chunk.clone()).unwrap();
            create_and_update_geometry(
                right,
                &mut own,
                meshes,
                texture_atlas_layout,
                ChunkFace::Right,
                mesh_handle,
                Res::clone(&blocks),
            );
        }
        if let Some(left) = self.get_neighbouring_chunk_mut(position, ChunkFace::Left) {
            let mesh_handle = left.mesh_handle.clone();
            let left = chunks.get_mut(left.chunk.clone()).unwrap();
            create_and_update_geometry(
                left,
                &mut own,
                meshes,
                texture_atlas_layout,
                ChunkFace::Left,
                mesh_handle,
                Res::clone(&blocks),
            );
        }

        meshes.insert(
            handle.clone(),
            own.gen_geometry(texture_atlas_layout, Res::clone(&blocks)),
        );

        let own_entity = self
            .chunks
            .get_mut(&position)
            .ok_or(ChunkError::ChunkNotFound)?;
        chunks.insert(own_handle.to_owned(), own);
        own_handle.clone_into(&mut own_entity.chunk);
        own_entity.mesh_handle = handle;
        Ok(())
    }

    /// Insert a [Chunk] and regenerate neighbours.
    /// Essentially it will regenerate meshes for all adjacent chunks
    /// This is needed as block face is only meshed if it exposed to air, and upon insertion a chunk
    /// assumes it is surrounded by air and will be updated by its neighbours
    #[allow(clippy::too_many_arguments)]
    pub fn insert_chunk_and_regenerate(
        &mut self,
        chunk: Chunk,
        position: IVec3,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        texture_atlas: Res<BlockAtlas>,
        chunks: &mut ResMut<Assets<Chunk>>,
        blocks: Res<Assets<Block>>,
    ) {
        self.insert_chunk(
            chunk,
            position,
            Res::clone(&texture_atlas),
            commands,
            meshes,
            materials,
            chunks,
            Res::clone(&blocks),
        );

        self.regenerate_chunk_at(
            position,
            meshes,
            texture_atlas.get_texture_atlas_layout(),
            chunks,
            blocks,
        )
        .unwrap();
    }

    pub fn set_block(
        &mut self,
        position: IVec3,
        block: Handle<Block>,
        blocks: Res<Assets<Block>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        texture_atlas_layout: &TextureAtlasLayout,
        chunks: &mut ResMut<Assets<Chunk>>,
    ) -> Result<(), ChunkError> {
        let chunk_coords = position / 16;
        let relative_coords = position - chunk_coords * 16;
        let chunk = self
            .chunks
            .get_mut(&(position / 16))
            .ok_or(ChunkError::ChunkNotFound)?;
        let index = ChunkShape::linearize([
            relative_coords.x as u32,
            relative_coords.y as u32,
            relative_coords.z as u32,
        ]);
        let chunk = chunks.get_mut(chunk.chunk.clone()).unwrap();
        chunk.blocks[index as usize] = block;
        self.regenerate_chunk_at(chunk_coords, meshes, texture_atlas_layout, chunks, blocks)?;
        Ok(())
    }
}
