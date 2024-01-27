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

    let next_unsolved_level_id = database
        .next_unsolved_level_id(0)
        .unwrap_or(database.max_level_id().unwrap());
    commands.insert_resource(LevelId(next_unsolved_level_id));
}

pub fn spawn_board(
    mut commands: Commands,
    database: Res<Database>,
    mut camera: Query<(&mut Transform, &mut MainCamera)>,
    window: Query<&Window>,
    board: Query<Entity, With<Board>>,
    level_id: Res<LevelId>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let database = database.lock().unwrap();
    let level = database.get_level_by_id(**level_id).unwrap();

    let spritesheet_handle = asset_server.load("textures/tilesheet.png");
    let tile_size = Vector2::new(128.0, 128.0);
    let texture_atlas = TextureAtlas::from_grid(
        spritesheet_handle,
        Vec2::new(tile_size.x, tile_size.y),
        6,
        3,
        Some(Vec2::new(1.0, 1.0)),
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let board_size = tile_size.x * level.dimensions.map(|x| x as f32);

    // move the camera to the center of the board
    let (mut transform, mut main_camera) = camera.single_mut();
    transform.translation.x = (board_size.x - tile_size.x) / 2.0;
    transform.translation.y = (board_size.y + tile_size.y) / 2.0;

    let window = window.single();
    let width_scale = board_size.x / window.resolution.width();
    let height_scale = board_size.y / window.resolution.height();
    let scale = if width_scale > height_scale {
        width_scale
    } else {
        height_scale
    };
    main_camera.target_scale = scale / 0.9;

    // despawn the previous `Board`
    commands.entity(board.single()).despawn_recursive();

    // spawn new `Board`
    let board = board::Board::with_level(level.clone());
    commands
        .spawn((Board { board, tile_size }, SpatialBundle::default()))
        .with_children(|parent| {
            for y in 0..level.dimensions.y {
                for x in 0..level.dimensions.x {
                    let position = Vector2::<i32>::new(x, y);
                    if level.get_unchecked(&position) == Tile::Void {
                        continue;
                    }
                    let tiles = HashMap::from([
                        (Tile::Floor, (0, 0.0)),
                        (Tile::Wall, (3, 1.0)),
                        (Tile::Crate, (1, 2.0)),
                        (Tile::Target, (2, 3.0)),
                        (Tile::Player, (0, 4.0)),
                    ]);
                    for (tile, (sprite_index, z_order)) in tiles.into_iter() {
                        if level.get_unchecked(&position).intersects(tile) {
                            let mut sprite = TextureAtlasSprite::new(sprite_index);
                            if settings.even_square_shades > 0.0
                                && tile == Tile::Floor
                                && (x + y) % 2 == 0
                            {
                                sprite.color = Color::WHITE * (1.0 - settings.even_square_shades);
                            }
                            let mut entity = parent.spawn((
                                SpriteSheetBundle {
                                    texture_atlas: texture_atlas_handle.clone(),
                                    sprite,
                                    transform: Transform::from_xyz(0.0, 0.0, z_order),
                                    ..default()
                                },
                                GridPosition(position),
                            ));
                            if tile == Tile::Player {
                                entity.insert((Player, AnimationState::default()));
                            } else if tile == Tile::Crate {
                                entity.insert(Crate);
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
    settings: Res<Settings>,
) {
    if !settings.auto_switch_to_next_unsolved_level {
        return;
    }
    let database = database.lock().unwrap();
    let board = &mut board.single_mut().board;
    debug_assert!(board.is_solved());
    info!("{}", "=".repeat(15));
    info!("#{} Sloved!", **level_id);
    info!("Moves   : {}", board.movements.move_count());
    info!("Pushes  : {}", board.movements.push_count());
    info!("Solution: {}", board.movements.lurd());
    database.update_solution(**level_id, &board.movements);
    switch_to_next_unsolved_level(&mut level_id, &database);
}

pub fn import_from_clipboard(level_id: &mut LevelId, database: &database::Database) {
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

pub fn switch_to_next_unsolved_level(level_id: &mut LevelId, database: &database::Database) {
    let next_unsolved_level_id = database
        .next_unsolved_level_id(level_id.0)
        .unwrap_or(level_id.0);
    level_id.0 = next_unsolved_level_id;
}

pub fn switch_to_previous_unsolved_level(level_id: &mut LevelId, database: &database::Database) {
    let next_unsolved_level_id = database
        .previous_unsolved_level_id(level_id.0)
        .unwrap_or(level_id.0);
    level_id.0 = next_unsolved_level_id;
}

pub fn switch_to_next_level(level_id: &mut LevelId, database: &database::Database) {
    if **level_id < database.max_level_id().unwrap() {
        **level_id += 1;
    }
}

pub fn switch_to_previous_level(level_id: &mut LevelId, database: &database::Database) {
    if **level_id > database.min_level_id().unwrap() {
        **level_id -= 1;
    }
}
