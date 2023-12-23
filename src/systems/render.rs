use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::resources::*;

use std::collections::HashSet;

pub fn setup_window(mut windows: Query<&mut Window>) {
    windows.single_mut().title = "Sokoban".to_string();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera::default()));
}

pub fn setup_version_info(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    commands.spawn(
        TextBundle::from_sections([TextSection::new(
            "version: ".to_string() + env!("CARGO_PKG_VERSION"),
            TextStyle {
                font_size: 14.0,
                color: Color::GRAY.with_a(ALPHA),
                ..default()
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
    );
}

pub fn setup_hud(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    let text_section = move |color, value: &str| {
        TextSection::new(
            value,
            TextStyle {
                font_size: 18.0,
                color,
                ..default()
            },
        )
    };
    commands.spawn((
        HUD,
        TextBundle::from_sections([
            text_section(Color::SEA_GREEN.with_a(ALPHA), "Level : "),
            text_section(Color::GOLD.with_a(ALPHA), ""),
            text_section(Color::SEA_GREEN.with_a(ALPHA), "\nMoves : "),
            text_section(Color::GOLD.with_a(ALPHA), ""),
            text_section(Color::SEA_GREEN.with_a(ALPHA), "\nPushes: "),
            text_section(Color::GOLD.with_a(ALPHA), ""),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
    ));
}

pub fn update_hud(
    mut hud: Query<&mut Text, With<HUD>>,
    board: Query<&Board>,
    level_id: Res<LevelId>,
) {
    let mut hud = hud.single_mut();
    let board = &board.single().board;

    hud.sections[1].value = format!("#{}", level_id.0);
    hud.sections[3].value = format!("{}", board.move_count());
    hud.sections[5].value = format!("{}", board.push_count());
}

// FIXME: 引入大量 BUG, 急需解决
// 问题出在玩家移动生效需要时间, 在此之前的算法会获取到过时的数据
pub fn animate_player_movement(
    mut player: Query<&mut GridPosition, With<Player>>,
    mut crates: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    mut player_movement: ResMut<PlayerMovement>,
    time: Res<Time>,
) {
    player_movement.timer.tick(time.delta());
    if !player_movement.timer.just_finished() {
        return;
    }

    let player_grid_position = &mut player.single_mut().0;
    if let Some(direction) = player_movement.directions.pop_back() {
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

pub fn animate_tiles_movement(
    mut tiles: Query<(&mut Transform, &GridPosition)>,
    board: Query<&Board>,
) {
    let Board { board, tile_size } = &board.single();
    for (mut transform, grid_position) in tiles.iter_mut() {
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

        let target_x = grid_position.0.x as f32 * tile_size.x;
        let target_y =
            board.level.size.y as f32 * tile_size.y - grid_position.0.y as f32 * tile_size.y;

        if (transform.translation.x - target_x).abs() > 0.001 {
            transform.translation.x = lerp(transform.translation.x, target_x, 0.5);
        } else {
            transform.translation.x = target_x;
        }
        if (transform.translation.y - target_y).abs() > 0.001 {
            transform.translation.y = lerp(transform.translation.y, target_y, 0.5);
        } else {
            transform.translation.y = target_y;
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

pub fn spawn_crate_reachable_marks(
    mut spawn_crate_reachable_marks_events: EventReader<SpawnCrateReachableMarks>,
    mut commands: Commands,
    crate_reachable: Res<CrateReachable>,
    board: Query<&Board>,
) {
    if spawn_crate_reachable_marks_events.is_empty() {
        return;
    }
    spawn_crate_reachable_marks_events.clear();

    let Board { board, tile_size } = board.single();

    if let CrateReachable::Some {
        selected_crate: _,
        came_from,
    } = crate_reachable.as_ref()
    {
        for position in came_from.keys() {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN.with_a(0.8),
                        custom_size: Some(Vec2::new(32.0, 32.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        position.x as f32 * tile_size.x,
                        (board.level.size.y - position.y) as f32 * tile_size.y,
                        10.0,
                    )),
                    ..default()
                },
                ReachableMark,
            ));
        }
    }
}

pub fn despawn_crate_reachable_marks(
    mut despawn_crate_reachable_marks_events: EventReader<DespawnCrateReachableMarks>,
    mut commands: Commands,
    reachable: Query<Entity, With<ReachableMark>>,
) {
    if despawn_crate_reachable_marks_events.is_empty() {
        return;
    }
    despawn_crate_reachable_marks_events.clear();

    reachable.for_each(|entity| commands.entity(entity).despawn());
}

pub fn update_grid_position_from_board(
    mut update_grid_position_events: EventReader<UpdateGridPositionEvent>,
    mut player: Query<&mut GridPosition, With<Player>>,
    mut crates: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    board: Query<&Board>,
) {
    if update_grid_position_events.is_empty() {
        return;
    }
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
            let crate_grid_position = &mut crate_grid_position.0;
            if *crate_grid_position == **old_position {
                *crate_grid_position = *new_position;
            }
        }
    }
}
