use benimator::{Animation, FrameRate};
use bevy::{prelude::*, window::WindowResized, winit::WinitWindows};
use nalgebra::Vector2;
use soukoban::{direction::Direction, Map};

use crate::{components::*, events::*, resources::*};

use std::{cmp::Ordering, collections::HashSet, time::Duration};

/// Sets the window icon for all windows
pub fn set_windows_icon(winit_windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/textures/box.png")
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
    commands.spawn((Name::new("Camera"), Camera2d, MainCamera::default()));
}

/// Animates the player character based on the player's movement and orientation.
pub fn animate_player(
    mut player: Query<(&mut AnimationState, &mut Sprite), With<Player>>,
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
    sprite.texture_atlas.as_mut().unwrap().index = animation_state.frame_index();
}

/// Handles player movement and interacts with boxes on the board.
pub fn handle_player_movement(
    mut player: Query<&mut GridPosition, With<Player>>,
    mut boxes: Query<&mut GridPosition, (With<Box>, Without<Player>)>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    time: Res<Time>,
    config: Res<Config>,
    mut box_enter_goal_events: EventWriter<BoxEnterGoal>,
    mut box_leave_goal_events: EventWriter<BoxLeaveGoal>,
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
            let occupied_goals_count = board
                .map
                .goal_positions()
                .intersection(board.map.box_positions())
                .count();
            board.do_action(direction);
            let new_occupied_goals_count = board
                .map
                .goal_positions()
                .intersection(board.map.box_positions())
                .count();
            match new_occupied_goals_count.cmp(&occupied_goals_count) {
                Ordering::Greater => drop(box_enter_goal_events.send_default()),
                Ordering::Less => drop(box_leave_goal_events.send_default()),
                _ => (),
            }

            player_grid_position.x += Into::<Vector2<i32>>::into(direction).x;
            player_grid_position.y += Into::<Vector2<i32>>::into(direction).y;

            let old_box_position = *player_grid_position;
            let new_box_position = old_box_position + &direction.into();
            let box_grid_positions: HashSet<_> = boxes.iter().map(|x| x.0).collect();
            if box_grid_positions.contains(player_grid_position) {
                for mut box_grid_position in boxes.iter_mut() {
                    if box_grid_position.0 == old_box_position {
                        box_grid_position.0 = new_box_position;
                    }
                }
            }
        }
    } else {
        while let Some(direction) = player_movement.directions.pop_back() {
            board.do_action(direction);

            player_grid_position.x += Into::<Vector2<i32>>::into(direction).x;
            player_grid_position.y += Into::<Vector2<i32>>::into(direction).y;

            let old_box_position = *player_grid_position;
            let new_box_position = old_box_position + &direction.into();
            let box_grid_positions: HashSet<_> = boxes.iter().map(|x| x.0).collect();
            if box_grid_positions.contains(player_grid_position) {
                for mut box_grid_position in boxes.iter_mut() {
                    if box_grid_position.0 == old_box_position {
                        box_grid_position.0 = new_box_position;
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

            let target_x = grid_position.x as f32 * tile_size.x as f32;
            let target_y = board.map.dimensions().y as f32 * tile_size.y as f32
                - grid_position.y as f32 * tile_size.y as f32;

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
            transform.translation.x = grid_position.x as f32 * tile_size.x as f32;
            transform.translation.y = board.map.dimensions().y as f32 * tile_size.y as f32
                - grid_position.y as f32 * tile_size.y as f32;
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
    mut boxes: Query<&mut GridPosition, (With<Box>, Without<Player>)>,
    board: Query<&Board>,
) {
    update_grid_position_events.clear();

    let map = &board.single().board.map;

    let player_grid_position = &mut player.single_mut().0;
    player_grid_position.x = map.player_position().x;
    player_grid_position.y = map.player_position().y;

    let box_grid_positions: HashSet<_> = boxes.iter().map(|x| x.0).collect();
    debug_assert!(box_grid_positions.difference(map.box_positions()).count() <= 1);
    if let Some(old_position) = box_grid_positions
        .difference(map.box_positions())
        .collect::<Vec<_>>()
        .first()
    {
        let new_position = *map
            .box_positions()
            .difference(&box_grid_positions)
            .collect::<Vec<_>>()
            .first()
            .unwrap();
        for mut box_grid_position in boxes.iter_mut() {
            let box_grid_position = &mut box_grid_position;
            if box_grid_position.0 == **old_position {
                box_grid_position.0 = *new_position;
            }
        }
    }
}

/// Adjust the camera scale to ensure the level is fully visible.
pub fn adjust_camera_scale(
    mut camera: Query<&mut MainCamera>,
    window: Query<&Window>,
    board: Query<&Board>,
    mut events: EventReader<WindowResized>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();

    camera.single_mut().target_scale =
        calculate_camera_default_scale(window.single(), &board.single().board.map);
}

/// Adjust the camera zoom to fit the entire board.
pub fn calculate_camera_default_scale(window: &Window, map: &Map) -> f32 {
    let tile_size = Vector2::new(128.0, 128.0);
    let board_size = tile_size.x as f32 * map.dimensions().map(|x| x as f32);

    let width_scale = board_size.x / window.resolution.width();
    let height_scale = board_size.y / window.resolution.height();
    let scale = if width_scale > height_scale {
        width_scale
    } else {
        height_scale
    };
    scale / 0.9
}
