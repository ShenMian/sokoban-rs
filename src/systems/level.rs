use arboard::Clipboard;
use bevy::prelude::*;
use nalgebra::Vector2;

use crate::board;
use crate::components::*;
use crate::database;
use crate::level::Level;
use crate::level::Tile;
use crate::resources::*;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub fn setup_database(mut commands: Commands) {
    let database = database::Database::from_file(Path::new("database.db"));
    database.initialize();
    info!("Loading levels from files");
    for path in fs::read_dir("assets/levels/").unwrap() {
        let path = path.unwrap().path();
        if !path.is_file() {
            continue;
        }
        info!("  {:?}", path);
        let levels = Level::load_from_file(&path).unwrap();
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
    commands.insert_resource(LevelId(1));
}

pub fn spawn_board(
    mut commands: Commands,
    database: Res<Database>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    board: Query<Entity, With<Board>>,
    level_id: Res<LevelId>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if !level_id.is_changed() {
        return;
    }

    let database = database.lock().unwrap();
    let level = database.get_level_by_id(**level_id).unwrap();

    let spritesheet_handle = asset_server.load("textures/spritesheet.png");
    let tile_size = Vector2::new(128.0, 128.0);
    let texture_atlas = TextureAtlas::from_grid(
        spritesheet_handle,
        Vec2::new(tile_size.x, tile_size.y),
        4,
        2,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let board_size = tile_size.x * level.dimensions.map(|x| x as f32);

    // move the camera to the center of the board
    let mut transform = camera.single_mut();
    transform.translation.x = board_size.x / 2.0;
    transform.translation.y = board_size.y / 2.0;

    // despawn the previous `Board`
    commands.entity(board.single()).despawn_recursive();

    // spawn new `Board`
    let board = board::Board::with_level(level.clone());
    commands
        .spawn((Board { board, tile_size }, SpatialBundle::default()))
        .with_children(|parent| {
            for y in 0..level.dimensions.y {
                for x in 0..level.dimensions.x {
                    let grid_position = Vector2::<i32>::new(x, y);
                    if level.get_unchecked(&grid_position) == Tile::Void {
                        continue;
                    }
                    let tiles = HashMap::from([
                        (Tile::Floor, 0),
                        (Tile::Wall, 1),
                        (Tile::Crate, 2),
                        (Tile::Target, 3),
                        (Tile::Player, 6),
                    ]);
                    for (tile, sprite_index) in tiles.into_iter() {
                        if level.get_unchecked(&grid_position).intersects(tile) {
                            let mut entity = parent.spawn((
                                SpriteSheetBundle {
                                    texture_atlas: texture_atlas_handle.clone(),
                                    sprite: TextureAtlasSprite::new(sprite_index),
                                    transform: Transform::from_xyz(0.0, 0.0, sprite_index as f32),
                                    ..default()
                                },
                                GridPosition(grid_position),
                            ));
                            if tile == Tile::Player {
                                entity.insert(Player);
                            } else if tile == Tile::Crate {
                                entity.insert(Crate);
                            }
                        }
                    }
                }
            }
        });
}

pub fn check_level_solved(
    mut board: Query<&mut Board>,
    mut level_id: ResMut<LevelId>,
    database: Res<Database>,
    player_movement: Res<PlayerMovement>,
) {
    if !player_movement.directions.is_empty() {
        return;
    }

    let database = database.lock().unwrap();
    let board = &mut board.single_mut().board;
    if board.is_solved() {
        info!("{}", "=".repeat(15));
        info!("#{} Sloved!", **level_id);
        info!("Moves   : {}", board.movements.move_count());
        info!("Pushes  : {}", board.movements.push_count());
        info!("Solution: {}", board.movements.lurd());
        database.update_solution(**level_id, &board.movements);
        switch_to_next_level(&mut level_id, &database);
    }
}

pub fn import_from_clipboard(
    level_id: &mut LevelId,
    database: &std::sync::MutexGuard<database::Database>,
) {
    let mut clipboard = Clipboard::new().unwrap();
    match Level::load_from_memory(clipboard.get_text().unwrap()) {
        Ok(levels) => {
            info!("import {} levels from clipboard", levels.len());
            database.import_levels(&levels);
            **level_id = database.get_level_id(&levels[0]).unwrap();
        }
        Err(msg) => error!("failed to import levels from clipboard: {}", msg),
    }
}

pub fn export_to_clipboard(board: &crate::board::Board) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard
        .set_text(board.level.export_map() + &board.level.export_metadata())
        .unwrap();
}

pub fn switch_to_next_level(
    level_id: &mut LevelId,
    database: &std::sync::MutexGuard<database::Database>,
) {
    if **level_id < database.max_level_id().unwrap() {
        **level_id += 1;
    }
}

pub fn switch_to_previous_level(
    level_id: &mut LevelId,
    database: &std::sync::MutexGuard<database::Database>,
) {
    if **level_id > database.min_level_id().unwrap() {
        **level_id -= 1;
    }
}
