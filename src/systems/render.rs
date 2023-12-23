use bevy::prelude::*;
use bevy::winit::WinitWindows;

use crate::components::*;
use crate::events::*;
use crate::level::PushState;
use crate::resources::*;

use std::collections::HashSet;

pub fn setup_window(mut window: Query<&mut Window>, winit_windows: NonSend<WinitWindows>) {
    let mut window = window.single_mut();
    window.title = "Sokoban".to_string();
    // window.mode = bevy::window::WindowMode::BorderlessFullscreen;

    // set window icon
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/textures/crate.png")
            .expect("Failed to open icon path")
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
            text_section(Color::SEA_GREEN.with_a(ALPHA), "\nBest moves : "),
            text_section(Color::GOLD.with_a(ALPHA), ""),
            text_section(Color::SEA_GREEN.with_a(ALPHA), "\nBest pushes: "),
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
    database: Res<Database>,
) {
    let mut hud = hud.single_mut();
    let board = &board.single().board;

    if level_id.is_changed() {
        hud.sections[1].value = format!("#{}", **level_id);

        let database = database.lock().unwrap();
        hud.sections[7].value =
            format!("{}", database.get_best_move_count(**level_id).unwrap_or(0));
        hud.sections[9].value =
            format!("{}", database.get_best_push_count(**level_id).unwrap_or(0));
    }

    hud.sections[3].value = format!("{}", board.movements.move_count());
    hud.sections[5].value = format!("{}", board.movements.push_count());
}

pub fn animate_player_movement(
    mut player: Query<&mut GridPosition, With<Player>>,
    mut crates: Query<&mut GridPosition, (With<Crate>, Without<Player>)>,
    mut player_movement: ResMut<PlayerMovement>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    if !settings.instant_move {
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
    } else {
        let player_grid_position = &mut player.single_mut().0;
        while let Some(direction) = player_movement.directions.pop_back() {
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
                transform.translation.x = lerp(transform.translation.x, target_x, 0.5);
            } else {
                transform.translation.x = target_x;
            }
            if (transform.translation.y - target_y).abs() > 0.001 {
                transform.translation.y = lerp(transform.translation.y, target_y, 0.5);
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

pub fn select_crate(
    mut select_crate_events: EventReader<SelectCrate>,
    mut commands: Commands,
    board: Query<&Board>,
    mut crate_reachable: ResMut<CrateReachable>,
) {
    if select_crate_events.is_empty() {
        return;
    }
    let event = select_crate_events.read().next().unwrap();

    let crate_position = &event.0;
    let Board { board, tile_size } = board.single();

    let path = board.level.crate_pushable_path(crate_position);

    // spawn crate reachable marks
    for &PushState {
        push_direction: _,
        crate_position,
    } in path.keys()
    {
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
            ReachableMark,
        ));
    }

    if path.is_empty() {
        *crate_reachable = CrateReachable::None;
    } else {
        *crate_reachable = CrateReachable::Some {
            selected_crate: *crate_position,
            path,
        };
    }
}

pub fn unselect_crate(
    mut unselect_crate_events: EventReader<UnselectCrate>,
    mut commands: Commands,
    reachable: Query<Entity, With<ReachableMark>>,
) {
    if unselect_crate_events.is_empty() {
        return;
    }
    unselect_crate_events.clear();

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
            let crate_grid_position = &mut crate_grid_position;
            if crate_grid_position.0 == **old_position {
                crate_grid_position.0 = *new_position;
            }
        }
    }
}
