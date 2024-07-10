use block_mesh::{Voxel, VoxelVisibility};

use crate::definition::Block;

impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        Block::get_voxel_visibility(self)
    }
}

impl Voxel for &Block {
    fn get_visibility(&self) -> VoxelVisibility {
        Block::get_voxel_visibility(self)
    }
}

impl Voxel for Box<Block> {
    fn get_visibility(&self) -> VoxelVisibility {
        Block::get_voxel_visibility(self)
    }
}
