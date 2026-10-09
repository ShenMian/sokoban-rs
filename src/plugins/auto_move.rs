#![allow(clippy::type_complexity)]

use bevy::{color::palettes::css::*, prelude::*};
use itertools::Itertools;
use soukoban::{
    deadlock::compute_static_deadlocks,
    path_finding::{compute_box_waypoints, compute_reachable_area},
    prelude::*,
};

use crate::{
    AppState,
    components::{Board, Box, GridPosition, Player},
    resources::AutoMoveState,
    systems::input::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::AutoMove), spawn_auto_move_marks)
        .add_systems(Update, mouse_input.run_if(in_state(AppState::AutoMove)))
        .add_systems(OnExit(AppState::AutoMove), cleanup_sprite_color);
    app.insert_resource(AutoMoveState::default());
}

/// Spawns auto-move reachable marks for boxes or the player.
pub fn spawn_auto_move_marks(
    mut commands: Commands,
    mut auto_move_state: ResMut<AutoMoveState>,
    board: Query<&Board>,
    mut boxes: Query<(&GridPosition, &mut Sprite), (With<Box>, Without<Player>)>,
    mut player: Query<&mut Sprite, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Board { board, tile_size } = board.single().unwrap();
    let map = &board.map;

    const MARK_COLOR: Srgba = LIME;
    const HIGHLIGHT_COLOR: Srgba = TURQUOISE;

    match &mut *auto_move_state {
        AutoMoveState::Box {
            position: box_position,
            waypoints,
        } => {
            let (computed_waypoints, _) =
                compute_box_waypoints(map, *box_position, soukoban::solver::Strategy::Quick);
            *waypoints = computed_waypoints;

            // remove static deadlock positions
            let static_deadlocks = compute_static_deadlocks(map);
            waypoints.retain(|state, _| !static_deadlocks.contains(&state.position));

            // spawn box pushable marks
            for mark_position in waypoints
                .keys()
                .map(|state| state.position)
                .filter(|position| position != box_position)
                .unique()
            {
                commands.spawn((
                    Name::new("Pushable mark"),
                    Sprite::from_color(
                        MARK_COLOR.with_alpha(0.8),
                        Vec2::new(tile_size.x as f32, tile_size.y as f32) / 4.0,
                    ),
                    Transform::from_xyz(
                        (mark_position.x * tile_size.x) as f32,
                        ((map.dimensions().y - mark_position.y) * tile_size.y) as f32,
                        10.0,
                    ),
                    DespawnOnExit(AppState::AutoMove),
                ));
            }

            if waypoints.is_empty() {
                next_state.set(AppState::Main);
                return;
            }

            // highlight selected box
            boxes
                .iter_mut()
                .filter(|(grid_position, ..)| grid_position.0 == *box_position)
                .for_each(|(_, mut sprite)| sprite.color = HIGHLIGHT_COLOR.into());
        }
        AutoMoveState::Player => {
            let mut compute_reachable_area =
                compute_reachable_area(map.player_position(), |position| {
                    !map[position].intersects(Tiles::Wall)
                        && !map.box_positions().contains(&position)
                });
            compute_reachable_area.remove(&map.player_position());

            // spawn player movable marks
            for mark_position in compute_reachable_area {
                commands.spawn((
                    Name::new("Movable mark"),
                    Sprite::from_color(
                        MARK_COLOR.with_alpha(0.8),
                        Vec2::new(tile_size.x as f32 / 4.0, tile_size.y as f32 / 4.0),
                    ),
                    Transform::from_xyz(
                        (mark_position.x * tile_size.x) as f32,
                        ((map.dimensions().y - mark_position.y) * tile_size.y) as f32,
                        10.0,
                    ),
                    DespawnOnExit(AppState::AutoMove),
                ));
            }

            // highlight selected player
            let mut sprite = player.single_mut().unwrap();
            sprite.color = HIGHLIGHT_COLOR.into();
        }
    }
}

/// Cleans up the visual representation of boxes or the player sprite.
pub fn cleanup_sprite_color(
    mut boxes: Query<(&GridPosition, &mut Sprite), (With<Box>, Without<Player>)>,
    mut player: Query<&mut Sprite, With<Player>>,
    auto_move_state: Res<AutoMoveState>,
) {
    match *auto_move_state {
        AutoMoveState::Box {
            position: box_position,
            ..
        } => {
            boxes
                .iter_mut()
                .filter(|(grid_position, _)| grid_position.0 == box_position)
                .for_each(|(_, mut sprite)| sprite.color = Color::WHITE);
        }
        AutoMoveState::Player => {
            let mut sprite = player.single_mut().unwrap();
            sprite.color = Color::WHITE;
        }
    }
}
