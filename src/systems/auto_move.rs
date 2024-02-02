use bevy::prelude::*;
use itertools::Itertools;

use crate::components::*;
use crate::level::Tile;
use crate::resources::*;
use crate::AppState;

pub fn spawn_auto_move_marks(
    mut commands: Commands,
    mut auto_move_state: ResMut<AutoMoveState>,
    board: Query<&Board>,
    mut crates: Query<(&GridPosition, &mut TextureAtlasSprite), (With<Crate>, Without<Player>)>,
    mut player: Query<&mut TextureAtlasSprite, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Board { board, tile_size } = board.single();

    const MARK_COLOR: Color = Color::GREEN;
    const HIGHLIGHT_COLOR: Color = Color::TURQUOISE;

    match &mut *auto_move_state {
        AutoMoveState::Crate {
            crate_position,
            paths,
        } => {
            *paths = board.level.crate_pushable_paths(crate_position);

            // spawn crate pushable marks
            for crate_position in paths.keys().map(|state| state.crate_position).unique() {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: MARK_COLOR.with_a(0.8),
                            custom_size: Some(Vec2::new(tile_size.x / 4.0, tile_size.y / 4.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            crate_position.x as f32 * tile_size.x,
                            (board.level.dimensions().y - crate_position.y) as f32 * tile_size.y,
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

            // highlight selected crate
            crates
                .iter_mut()
                .filter(|(grid_position, _)| ***grid_position == *crate_position)
                .for_each(|(_, mut sprite)| sprite.color = HIGHLIGHT_COLOR);
        }
        AutoMoveState::Player => {
            let mut reachable_area =
                board
                    .level
                    .reachable_area(&board.level.player_position, |position| {
                        board.level.get_unchecked(position).intersects(Tile::Wall)
                            || board.level.crate_positions.contains(position)
                    });
            reachable_area.remove(&board.level.player_position);

            // spawn player movable marks
            for crate_position in reachable_area {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: MARK_COLOR.with_a(0.8),
                            custom_size: Some(Vec2::new(tile_size.x / 4.0, tile_size.y / 4.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            crate_position.x as f32 * tile_size.x,
                            (board.level.dimensions().y - crate_position.y) as f32 * tile_size.y,
                            10.0,
                        ),
                        ..default()
                    },
                    PlayerMovableMark,
                ));
            }

            // highlight selected player
            let mut sprite = player.single_mut();
            sprite.color = HIGHLIGHT_COLOR;
        }
    }
}

pub fn despawn_auto_move_marks(
    mut commands: Commands,
    mut crates: Query<(&GridPosition, &mut TextureAtlasSprite), (With<Crate>, Without<Player>)>,
    mut player: Query<&mut TextureAtlasSprite, With<Player>>,
    marks: Query<Entity, Or<(With<CratePushableMark>, With<PlayerMovableMark>)>>,
    auto_move_state: Res<AutoMoveState>,
) {
    match *auto_move_state {
        AutoMoveState::Crate { crate_position, .. } => {
            crates
                .iter_mut()
                .filter(|(grid_position, _)| ***grid_position == crate_position)
                .for_each(|(_, mut sprite)| sprite.color = Color::WHITE);
        }
        AutoMoveState::Player => {
            let mut sprite = player.single_mut();
            sprite.color = Color::WHITE;
        }
    }

    marks.for_each(|entity| commands.entity(entity).despawn());
}
