use bevy::{color::palettes::css::*, prelude::*};
use itertools::Itertools;
use soukoban::{deadlock::calculate_static_deadlocks, path_finding::reachable_area, Tiles};

use crate::{box_pushable_paths, components::*, resources::*, AppState};

/// Spawns auto-move reachable marks for boxes or the player.
pub fn spawn_auto_move_marks(
    mut commands: Commands,
    mut auto_move_state: ResMut<AutoMoveState>,
    board: Query<&Board>,
    mut boxes: Query<(&GridPosition, &mut Sprite), (With<Box>, Without<Player>)>,
    mut player: Query<&mut Sprite, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Board { board, tile_size } = board.single();

    const MARK_COLOR: Srgba = LIME;
    const HIGHLIGHT_COLOR: Srgba = TURQUOISE;

    match &mut *auto_move_state {
        AutoMoveState::Box {
            position: box_position,
            paths,
        } => {
            *paths = box_pushable_paths(&board.level, box_position);

            // remove static deadlock positions
            let static_deadlocks = calculate_static_deadlocks(&board.level);
            paths.retain(|state, _| !static_deadlocks.contains(&state.box_position));

            // spawn box pushable marks
            for box_position in paths.keys().map(|state| state.box_position).unique() {
                commands.spawn((
                    StateScoped(AppState::AutoMove),
                    SpriteBundle {
                        sprite: Sprite {
                            color: MARK_COLOR.with_alpha(0.8).into(),
                            custom_size: Some(Vec2::new(
                                tile_size.x as f32 / 4.0,
                                tile_size.y as f32 / 4.0,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            box_position.x as f32 * tile_size.x as f32,
                            (board.level.dimensions().y - box_position.y) as f32
                                * tile_size.y as f32,
                            10.0,
                        ),
                        ..default()
                    },
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
            let mut reachable_area = reachable_area(board.level.player_position(), |position| {
                !board.level[position].intersects(Tiles::Wall)
                    && !board.level.box_positions().contains(&position)
            });
            reachable_area.remove(&board.level.player_position());

            // spawn player movable marks
            for box_position in reachable_area {
                commands.spawn((
                    StateScoped(AppState::AutoMove),
                    SpriteBundle {
                        sprite: Sprite {
                            color: MARK_COLOR.with_alpha(0.8).into(),
                            custom_size: Some(Vec2::new(
                                tile_size.x as f32 / 4.0,
                                tile_size.y as f32 / 4.0,
                            )),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            box_position.x as f32 * tile_size.x as f32,
                            (board.level.dimensions().y - box_position.y) as f32
                                * tile_size.y as f32,
                            10.0,
                        ),
                        ..default()
                    },
                ));
            }

            // highlight selected player
            let mut sprite = player.single_mut();
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
            let mut sprite = player.single_mut();
            sprite.color = Color::WHITE;
        }
    }
}
