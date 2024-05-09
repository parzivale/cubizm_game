use bevy::app::App;
use bevy::prelude::*;

pub use block::*;
pub use chunk::*;
pub use chunks::*;
pub use util::*;

mod block;
mod chunk;
mod chunks;
mod util;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    BlocksLoaded,
    ChunksLoaded,
    Finished,
}

pub struct Cubizm;
impl Plugin for Cubizm {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
