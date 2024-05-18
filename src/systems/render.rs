use benimator::{Animation, FrameRate};
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use nalgebra::Vector2;

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use soukoban::direction::Direction;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::time::Duration;

/// Sets the window icon for all windows
pub fn set_windows_icon(winit_windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/textures/crate.png")
            .expect("failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();
    for window in winit_windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

/// Sets up the main 2D camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera::default()));
}

/// Animates the player character based on the player's movement and orientation.
pub fn animate_player(
    mut player: Query<(&mut AnimationState, &mut TextureAtlas), With<Player>>,
    mut board: Query<&mut Board>,
    time: Res<Time>,
    player_movement: Res<PlayerMovement>,
) {
    let board = &mut board.single_mut().board;
    let (animation_state, sprite) = &mut player.single_mut();

    // TODO: The character's orientation looks a bit weird when just doing
    // push-undo. Supporting action-undo should fix that.
    let player_orientation = board.player_orientation();

    let face_up = Animation::from_indices([9], FrameRate::from_frame_duration(Duration::MAX));
    let face_down = Animation::from_indices([6], FrameRate::from_frame_duration(Duration::MAX));
    let face_left = Animation::from_indices([15], FrameRate::from_frame_duration(Duration::MAX));
    let face_right = Animation::from_indices([12], FrameRate::from_frame_duration(Duration::MAX));

    let move_up = Animation::from_indices(9..=9 + 2, FrameRate::from_fps(6.0));
    let move_down = Animation::from_indices(6..=6 + 2, FrameRate::from_fps(6.0));
    let move_left = Animation::from_indices(15..=15 + 2, FrameRate::from_fps(6.0));
    let move_right = Animation::from_indices(12..=12 + 2, FrameRate::from_fps(6.0));

    let animation = if player_movement.directions.is_empty() {
        match player_orientation {
            Direction::Up => face_up,
            Direction::Right => face_right,
            Direction::Down => face_down,
            Direction::Left => face_left,
        }
    } else {
        match player_orientation {
            Direction::Up => move_up,
            Direction::Right => move_right,
            Direction::Down => move_down,
            Direction::Left => move_left,
        }
    };

    animation_state.update(&animation, time.delta());
    sprite.index = animation_state.frame_index();
}

/// Handles player movement and interacts with crates on the board.
pub fn handle_player_movement(
    mut player: Query<&mut GridPosition, With<Player>>,
    mut crates: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    time: Res<Time>,
    config: Res<Config>,
    mut crate_enter_target_events: EventWriter<CrateEnterTarget>,
    mut crate_leave_target_events: EventWriter<CrateLeaveTarget>,
    mut level_solved_events: EventWriter<LevelSolved>,
) {
    if player_movement.directions.is_empty() {
        return;
    }

    let board = &mut board.single_mut().board;

    let player_grid_position = &mut **player.single_mut();
    if !config.instant_move {
        player_movement.timer.tick(time.delta());
        if !player_movement.timer.just_finished() {
            return;
        }
        if let Some(direction) = player_movement.directions.pop_back() {
            let occupied_targets_count = board
                .level
                .target_positions
                .intersection(&board.level.crate_positions)
                .count();
            board.move_or_push(direction);
            let new_occupied_targets_count = board
                .level
                .target_positions
                .intersection(&board.level.crate_positions)
                .count();
            match new_occupied_targets_count.cmp(&occupied_targets_count) {
                Ordering::Greater => drop(crate_enter_target_events.send_default()),
                Ordering::Less => drop(crate_leave_target_events.send_default()),
                _ => (),
            }

            player_grid_position.x += Into::<Vector2<i32>>::into(direction).x;
            player_grid_position.y += Into::<Vector2<i32>>::into(direction).y;

            let old_crate_position = *player_grid_position;
            let new_crate_position = old_crate_position + &direction.into();
            let crate_grid_positions: HashSet<_> = crates.iter().map(|x| x.0).collect();
            if crate_grid_positions.contains(player_grid_position) {
                for mut crate_grid_position in crates.iter_mut() {
                    if crate_grid_position.0 == old_crate_position {
                        crate_grid_position.0 = new_crate_position;
                    }
                }
            }
        }
    } else {
        while let Some(direction) = player_movement.directions.pop_back() {
            board.move_or_push(direction);

            player_grid_position.x += Into::<Vector2<i32>>::into(direction).x;
            player_grid_position.y += Into::<Vector2<i32>>::into(direction).y;

            let old_crate_position = *player_grid_position;
            let new_crate_position = old_crate_position + &direction.into();
            let crate_grid_positions: HashSet<_> = crates.iter().map(|x| x.0).collect();
            if crate_grid_positions.contains(player_grid_position) {
                for mut crate_grid_position in crates.iter_mut() {
                    if crate_grid_position.0 == old_crate_position {
                        crate_grid_position.0 = new_crate_position;
                    }
                }
            }
        }
    }

    if board.is_solved() {
        level_solved_events.send_default();
    }
}

/// Applies smooth motion to tiles based on their grid positions.
pub fn smooth_tile_motion(
    mut tiles: Query<(&mut Transform, &GridPosition)>,
    board: Query<&Board>,
    config: Res<Config>,
) {
    let Board { board, tile_size } = &board.single();
    for (mut transform, grid_position) in tiles.iter_mut() {
        if !config.instant_move {
            let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

            let target_x = grid_position.x as f32 * tile_size.x;
            let target_y = board.level.dimensions().y as f32 * tile_size.y
                - grid_position.y as f32 * tile_size.y;

            if (transform.translation.x - target_x).abs() > 0.001 {
                transform.translation.x = lerp(transform.translation.x, target_x, 0.3);
            } else {
                transform.translation.x = target_x;
            }
            if (transform.translation.y - target_y).abs() > 0.001 {
                transform.translation.y = lerp(transform.translation.y, target_y, 0.3);
            } else {
                transform.translation.y = target_y;
            }
        } else {
            transform.translation.x = grid_position.x as f32 * tile_size.x;
            transform.translation.y = board.level.dimensions().y as f32 * tile_size.y
                - grid_position.y as f32 * tile_size.y;
        }
    }
}

/// Applies smooth motion to the main camera.
pub fn smooth_camera_motion(
    camera: Query<&MainCamera>,
    mut projection: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let main_camera = camera.single();
    let mut projection = projection.single_mut();

    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    if (projection.scale - main_camera.target_scale).abs() > 0.001 {
        projection.scale = lerp(projection.scale, main_camera.target_scale, 0.3);
    } else {
        projection.scale = main_camera.target_scale;
    }
}

/// Updates grid positions of entities based on the board.
pub fn update_grid_position_from_board(
    mut update_grid_position_events: EventReader<UpdateGridPositionEvent>,
    mut player: Query<&mut GridPosition, With<Player>>,
    mut crates: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    board: Query<&Board>,
) {
    update_grid_position_events.clear();

    let board = &board.single().board;

    let player_grid_position = &mut player.single_mut().0;
    player_grid_position.x = board.level.player_position.x;
    player_grid_position.y = board.level.player_position.y;

    let crate_grid_positions: HashSet<_> = crates.iter().map(|x| x.0).collect();
    debug_assert!(
        crate_grid_positions
            .difference(&board.level.crate_positions)
            .count()
            <= 1
    );
    if let Some(old_position) = crate_grid_positions
        .difference(&board.level.crate_positions)
        .collect::<Vec<_>>()
        .first()
    {
        let new_position = *board
            .level
            .crate_positions
            .difference(&crate_grid_positions)
            .collect::<Vec<_>>()
            .first()
            .unwrap();
        for mut crate_grid_position in crates.iter_mut() {
            let crate_grid_position = &mut crate_grid_position;
            if crate_grid_position.0 == **old_position {
                crate_grid_position.0 = *new_position;
            }
        }
    }
}
