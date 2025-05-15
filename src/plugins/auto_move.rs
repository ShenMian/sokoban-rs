#![allow(clippy::type_complexity)]

use bevy::{color::palettes::css::*, prelude::*};
use itertools::Itertools;
use soukoban::{deadlock::calculate_static_deadlocks, path_finding::reachable_area, Tiles};

use crate::{
    box_pushable_paths,
    components::{Board, Box, GridPosition, Player},
    resources::AutoMoveState,
    systems::input::*,
    AppState,
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
            paths,
        } => {
            *paths = box_pushable_paths(map, box_position);

            // remove static deadlock positions
            let static_deadlocks = calculate_static_deadlocks(map);
            paths.retain(|state, _| !static_deadlocks.contains(&state.box_position));

            // spawn box pushable marks
            for box_position in paths.keys().map(|state| state.box_position).unique() {
                commands.spawn((
                    Name::new("Pushable mark"),
                    Sprite::from_color(
                        MARK_COLOR.with_alpha(0.8),
                        Vec2::new(tile_size.x as f32 / 4.0, tile_size.y as f32 / 4.0),
                    ),
                    Transform::from_xyz(
                        box_position.x as f32 * tile_size.x as f32,
                        (map.dimensions().y - box_position.y) as f32 * tile_size.y as f32,
                        10.0,
                    ),
                    StateScoped(AppState::AutoMove),
                ));
            }

            if paths.is_empty() {
                next_state.set(AppState::Main);
                return;
            }

            // highlight selected box
            boxes
                .iter_mut()
                .filter(|(grid_position, ..)| ***grid_position == *box_position)
                .for_each(|(_, mut sprite)| sprite.color = HIGHLIGHT_COLOR.into());
        }
        AutoMoveState::Player => {
            let mut reachable_area = reachable_area(map.player_position(), |position| {
                !map[position].intersects(Tiles::Wall) && !map.box_positions().contains(&position)
            });
            reachable_area.remove(&map.player_position());

            // spawn player movable marks
            for box_position in reachable_area {
                commands.spawn((
                    Name::new("Movable mark"),
                    Sprite::from_color(
                        MARK_COLOR.with_alpha(0.8),
                        Vec2::new(tile_size.x as f32 / 4.0, tile_size.y as f32 / 4.0),
                    ),
                    Transform::from_xyz(
                        box_position.x as f32 * tile_size.x as f32,
                        (map.dimensions().y - box_position.y) as f32 * tile_size.y as f32,
                        10.0,
                    ),
                    StateScoped(AppState::AutoMove),
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
                .filter(|(grid_position, _)| ***grid_position == box_position)
                .for_each(|(_, mut sprite)| sprite.color = Color::WHITE);
        }
        AutoMoveState::Player => {
            let mut sprite = player.single_mut().unwrap();
            sprite.color = Color::WHITE;
        }
    }
}
