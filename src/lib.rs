use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use cubizm_block::BlockPlugin;
use cubizm_chunks::ChunksPlugin;
use cubizm_core::Cubizm;

pub struct CubizmGameDefault;

impl PluginGroup for CubizmGameDefault {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BlockPlugin)
            .add(ChunksPlugin)
            .add(Cubizm)
    }
}
