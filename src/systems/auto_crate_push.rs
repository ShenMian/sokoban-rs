use bevy::prelude::*;
use itertools::Itertools;

use crate::components::*;
use crate::resources::*;
use crate::AppState;

pub fn spawn_crate_pushable_marks(
    mut commands: Commands,
    mut auto_crate_push_state: ResMut<AutoCratePushState>,
    board: Query<&Board>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let crate_position = &auto_crate_push_state.selected_crate;
    let Board { board, tile_size } = board.single();

    let paths = board.level.crate_pushable_paths(crate_position);

    // spawn crate pushable marks
    for crate_position in paths.keys().map(|state| state.crate_position).unique() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN.with_a(0.8),
                    custom_size: Some(Vec2::new(tile_size.x / 4.0, tile_size.y / 4.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    crate_position.x as f32 * tile_size.x,
                    (board.level.dimensions.y - crate_position.y) as f32 * tile_size.y,
                    10.0,
                ),
                ..default()
            },
            CratePushableMark,
        ));
    }

    if paths.is_empty() {
        next_state.set(AppState::Main);
        return;
    }

    auto_crate_push_state.paths = paths;
}

pub fn despawn_crate_pushable_marks(
    mut commands: Commands,
    marks: Query<Entity, With<CratePushableMark>>,
) {
    marks.for_each(|entity| commands.entity(entity).despawn());
}