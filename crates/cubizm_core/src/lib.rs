use std::f32::consts::PI;

use bevy::app::App;
use bevy::prelude::*;

pub use util::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    Setup,
    BlocksLoaded,
    ChunksLoaded,
    Finished,
}

mod util;

pub struct Cubizm;
impl Plugin for Cubizm {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        app.add_systems(OnEnter(AppState::ChunksLoaded), finish);
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn finish(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Finished);
}
