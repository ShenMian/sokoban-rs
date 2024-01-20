use bevy::prelude::*;
use bevy::winit::WinitWindows;

use crate::components::*;
use crate::direction::Direction;
use crate::events::*;
use crate::resources::*;

use std::collections::HashSet;

pub fn setup_window(mut window: Query<&mut Window>, winit_windows: NonSend<WinitWindows>) {
    let mut window = window.single_mut();
    window.title = "Sokoban".to_string();
    // window.mode = bevy::window::WindowMode::BorderlessFullscreen;

    // set window icon
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

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera::default()));
}

pub fn animate_player(
    mut player: Query<(&mut AnimationState, &mut TextureAtlasSprite), With<Player>>,
    mut board: Query<&mut Board>,
    time: Res<Time>,
    player_movement: Res<PlayerMovement>,
) {
    let board = &mut board.single_mut().board;
    let (animation_state, sprite) = &mut player.single_mut();

    // TODO: 仅进行推动撤回时角色的朝向正确, 但看上去有点怪. 支持操作撤回应该可以解决该问题
    let direction = board.player_facing_direction();

    let face_up = benimator::Animation::from_indices([55], benimator::FrameRate::from_fps(1.0));
    let face_down = benimator::Animation::from_indices([52], benimator::FrameRate::from_fps(1.0));
    let face_left = benimator::Animation::from_indices([81], benimator::FrameRate::from_fps(1.0));
    let face_right = benimator::Animation::from_indices([78], benimator::FrameRate::from_fps(1.0));

    let move_up = benimator::Animation::from_indices(55..=57, benimator::FrameRate::from_fps(6.0));
    let move_down =
        benimator::Animation::from_indices(52..=54, benimator::FrameRate::from_fps(6.0));
    let move_left =
        benimator::Animation::from_indices(81..=83, benimator::FrameRate::from_fps(6.0));
    let move_right =
        benimator::Animation::from_indices(78..=80, benimator::FrameRate::from_fps(6.0));

    let animation;
    if player_movement.directions.is_empty() {
        animation = match direction {
            Direction::Up => face_up,
            Direction::Right => face_right,
            Direction::Down => face_down,
            Direction::Left => face_left,
        };
    } else {
        animation = match direction {
            Direction::Up => move_up,
            Direction::Right => move_right,
            Direction::Down => move_down,
            Direction::Left => move_left,
        };
    }

    animation_state.update(&animation, time.delta());
    sprite.index = animation_state.frame_index();
}

pub fn animate_player_movement(
    mut player: Query<&mut GridPosition, With<Player>>,
    mut crates: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    mut board: Query<&mut Board>,
    mut player_movement: ResMut<PlayerMovement>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    let board = &mut board.single_mut().board;

    let player_grid_position = &mut **player.single_mut();
    if !settings.instant_move {
        player_movement.timer.tick(time.delta());
        if !player_movement.timer.just_finished() {
            return;
        }
        if let Some(direction) = player_movement.directions.pop_back() {
            board.move_or_push(direction);

            player_grid_position.x += direction.to_vector().x;
            player_grid_position.y += direction.to_vector().y;

            let old_crate_position = *player_grid_position;
            let new_crate_position = old_crate_position + direction.to_vector();
            let crate_grid_positions: HashSet<_> = crates.iter().map(|x| x.0).collect();
            if crate_grid_positions.contains(&player_grid_position) {
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

            player_grid_position.x += direction.to_vector().x;
            player_grid_position.y += direction.to_vector().y;

            let old_crate_position = *player_grid_position;
            let new_crate_position = old_crate_position + direction.to_vector();
            let crate_grid_positions: HashSet<_> = crates.iter().map(|x| x.0).collect();
            if crate_grid_positions.contains(&player_grid_position) {
                for mut crate_grid_position in crates.iter_mut() {
                    if crate_grid_position.0 == old_crate_position {
                        crate_grid_position.0 = new_crate_position;
                    }
                }
            }
        }
    }
}

pub fn animate_tiles_movement(
    mut tiles: Query<(&mut Transform, &GridPosition)>,
    board: Query<&Board>,
    settings: Res<Settings>,
) {
    let Board { board, tile_size } = &board.single();
    for (mut transform, grid_position) in tiles.iter_mut() {
        if !settings.instant_move {
            let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

            let target_x = grid_position.x as f32 * tile_size.x;
            let target_y = board.level.dimensions.y as f32 * tile_size.y
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
            transform.translation.y = board.level.dimensions.y as f32 * tile_size.y
                - grid_position.y as f32 * tile_size.y;
        }
    }
}

pub fn animate_camera_zoom(
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
