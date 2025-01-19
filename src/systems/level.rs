use arboard::Clipboard;
use bevy::{color::palettes::css::*, prelude::*};
use nalgebra::Vector2;
use soukoban::{Level, Tiles};

use crate::{board, calculate_camera_default_scale, components::*, database, resources::*};

use std::{collections::HashMap, fs, path::Path, sync::Mutex};

/// Sets up the database, initializes it, and loads levels from files into the database.
pub fn setup_database(mut commands: Commands) {
    let database = database::Database::from_file(Path::new("db.sqlite3"));
    database.initialize();
    info!("Loading levels from files");
    for path in fs::read_dir("assets/levels/").unwrap() {
        let path = path.unwrap().path();
        if !path.is_file() {
            continue;
        }
        info!("  {:?}", path);
        let levels: Vec<_> = Level::load_from_str(&fs::read_to_string(path).unwrap())
            .filter_map(Result::ok)
            .collect();
        database.import_levels(&levels);
    }
    info!("Done");
    commands.insert_resource(Database(Mutex::new(database)));
}

pub fn setup_level(mut commands: Commands, database: Res<Database>) {
    let database = database.lock().unwrap();
    let level = database.get_level_by_id(1).unwrap();
    commands.spawn(Board {
        board: board::Board::with_level(level),
        tile_size: Vector2::zeros(),
    });

    let next_unsolved_level_id = database
        .next_unsolved_level_id(0)
        .unwrap_or(database.max_level_id().unwrap());
    commands.insert_resource(LevelId(next_unsolved_level_id));
}

pub fn spawn_board(
    mut commands: Commands,
    database: Res<Database>,
    mut player_movement: ResMut<PlayerMovement>,
    mut camera: Query<(&mut Transform, &mut MainCamera)>,
    window: Query<&Window>,
    board: Query<Entity, With<Board>>,
    level_id: Res<LevelId>,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
    mut spritesheet_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    player_movement.directions.clear();

    let database = database.lock().unwrap();
    let level = database.get_level_by_id(level_id.0).unwrap();

    let spritesheet_handle = asset_server.load("textures/tilesheet.png");
    let tile_size = Vector2::new(128, 128);
    let spritesheet_layout = TextureAtlasLayout::from_grid(
        UVec2::new(tile_size.x, tile_size.y),
        6,
        3,
        Some(UVec2::new(1, 1)),
        None,
    );
    let spritesheet_layout_handle = spritesheet_layouts.add(spritesheet_layout);

    let board_size = Vector2::new(
        tile_size.x * level.map().dimensions().x as u32,
        tile_size.y * level.map().dimensions().y as u32,
    );

    // move the camera to the center of the board
    let (mut transform, mut main_camera) = camera.single_mut();
    transform.translation.x = (board_size.x - tile_size.x) as f32 / 2.0;
    transform.translation.y = (board_size.y + tile_size.y) as f32 / 2.0;

    main_camera.target_scale = calculate_camera_default_scale(window.single(), &level);

    // despawn the previous `Board`
    commands.entity(board.single()).despawn_recursive();

    // spawn new `Board`
    let board = board::Board::with_level(level.clone());
    commands
        .spawn((
            Name::new("Board"),
            Board { board, tile_size },
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            for y in 0..level.map().dimensions().y {
                for x in 0..level.map().dimensions().x {
                    let position = Vector2::<i32>::new(x, y);
                    if level.map()[position].is_empty() {
                        continue;
                    }
                    let tiles = HashMap::from([
                        (Tiles::Floor, (0, 0.0)),
                        (Tiles::Wall, (3, 1.0)),
                        (Tiles::Box, (1, 2.0)),
                        (Tiles::Goal, (2, 3.0)),
                        (Tiles::Player, (0, 4.0)),
                    ]);
                    for (tile, (sprite_index, z_order)) in tiles.into_iter() {
                        if level.map()[position].intersects(tile) {
                            let mut sprite = Sprite::default();
                            if config.even_square_shades > 0.0
                                && tile == Tiles::Floor
                                && (x + y) % 2 == 0
                            {
                                sprite.color = (WHITE * (1.0 - config.even_square_shades)).into();
                            }
                            let mut entity = parent.spawn((
                                Sprite {
                                    image: spritesheet_handle.clone(),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: spritesheet_layout_handle.clone(),
                                        index: sprite_index,
                                    }),
                                    ..default()
                                },
                                Transform::from_xyz(0.0, 0.0, z_order),
                                GridPosition(position),
                            ));
                            if tile == Tiles::Player {
                                entity.insert((Player, AnimationState::default()));
                            } else if tile == Tiles::Box {
                                entity.insert(Box);
                            }
                        }
                    }
                }
            }
        });
}

pub fn auto_switch_to_next_unsolved_level(
    mut board: Query<&mut Board>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    config: Res<Config>,
) {
    if !config.auto_switch_to_next_unsolved_level {
        return;
    }
    let database = database.lock().unwrap();
    let board = &mut board.single_mut().board;
    debug_assert!(board.is_solved());
    info!("{}", "=".repeat(15));
    info!("#{} Solved!", level_id.0);
    info!("Moves   : {}", board.actions().moves());
    info!("Pushes  : {}", board.actions().pushes());
    info!("Solution: {}", board.actions().to_string());
    database.update_solution(level_id.0, board.actions());
    switch_to_next_unsolved_level(&mut level_id, &database);
}

/// Imports levels from the system clipboard.
pub fn import_from_clipboard(level_id: &mut LevelId, database: &database::Database) {
    let mut clipboard = Clipboard::new().unwrap();
    match Level::load_from_str(&clipboard.get_text().unwrap()).collect::<Result<Vec<_>, _>>() {
        Ok(levels) => {
            if levels.is_empty() {
                error!("failed to import any level from clipboard");
                return;
            }
            info!("import {} levels from clipboard", levels.len());
            database.import_levels(&levels);
            level_id.0 = database.get_level_id(&levels[0]).unwrap();
        }
        Err(msg) => error!("failed to import levels from clipboard: {}", msg),
    }
}

pub fn export_to_clipboard(board: &crate::board::Board) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(board.level.to_string()).unwrap();
}

/// Switches to the next unsolved level based on the current level ID.
pub fn switch_to_next_unsolved_level(level_id: &mut LevelId, database: &database::Database) {
    let next_unsolved_level_id = database
        .next_unsolved_level_id(level_id.0)
        .unwrap_or(level_id.0);
    level_id.0 = next_unsolved_level_id;
}

/// Switches to the previous unsolved level based on the current level ID.
pub fn switch_to_previous_unsolved_level(level_id: &mut LevelId, database: &database::Database) {
    let next_unsolved_level_id = database
        .previous_unsolved_level_id(level_id.0)
        .unwrap_or(level_id.0);
    level_id.0 = next_unsolved_level_id;
}

/// Switches to the next level based on the current level ID.
pub fn switch_to_next_level(level_id: &mut LevelId, database: &database::Database) {
    if level_id.0 < database.max_level_id().unwrap() {
        level_id.0 += 1;
    }
}

/// Switches to the previous level based on the current level ID.
pub fn switch_to_previous_level(level_id: &mut LevelId, database: &database::Database) {
    if level_id.0 > database.min_level_id().unwrap() {
        level_id.0 -= 1;
    }
}
