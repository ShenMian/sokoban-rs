use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::input::Action;

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

const BUTTON_NORMAL_COLOR: Color = Color::rgba(0.7, 0.7, 0.7, 0.8);
const BUTTON_HOVERED_COLOR: Color = Color::rgba(0.55, 0.55, 0.55, 0.8);
const BUTTON_PRESSED_COLOR: Color = Color::rgba(0.35, 0.75, 0.35, 0.8);

#[derive(Component, PartialEq)]
pub enum MainButton {
    InstantMove,
    AutomaticSolution,
    PreviousLevel,
    NextLevel,
}

pub fn button_input_to_action(
    buttons: Query<(&Interaction, &MainButton), (Changed<Interaction>, With<Button>)>,
    mut action_state: ResMut<ActionState<Action>>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *button {
            MainButton::InstantMove => action_state.press(Action::InstantMove),
            MainButton::AutomaticSolution => action_state.press(Action::AutomaticSolution),
            MainButton::PreviousLevel => action_state.press(Action::PreviousLevel),
            MainButton::NextLevel => action_state.press(Action::NextLevel),
        }
    }
}

pub fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    MainButton::InstantMove,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        border_color: Color::NONE.into(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        image: asset_server.load("textures/instant_move_off.png").into(),
                        ..default()
                    });
                });
            parent
                .spawn((
                    MainButton::AutomaticSolution,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        border_color: Color::NONE.into(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        image: asset_server.load("textures/automatic_solution.png").into(),
                        ..default()
                    });
                });
            parent
                .spawn((
                    MainButton::PreviousLevel,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        border_color: Color::NONE.into(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        image: asset_server.load("textures/previous.png").into(),
                        ..default()
                    });
                });
            parent
                .spawn((
                    MainButton::NextLevel,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        border_color: Color::NONE.into(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        image: asset_server.load("textures/next.png").into(),
                        ..default()
                    });
                });
        });
}

pub fn update_button_state(
    mut buttons: Query<(&MainButton, &Children), With<Button>>,
    mut image: Query<&mut UiImage>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
) {
    if !settings.is_changed() {
        return;
    }
    for (button, children) in &mut buttons {
        if *button == MainButton::InstantMove {
            let mut image = image.get_mut(children[0]).unwrap();
            image.texture = if settings.instant_move {
                asset_server.load("textures/instant_move_on.png").into()
            } else {
                asset_server.load("textures/instant_move_off.png").into()
            };
        }
    }
}

pub fn button_visual_effect(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => *color = BUTTON_PRESSED_COLOR.into(),
            Interaction::Hovered => *color = BUTTON_HOVERED_COLOR.into(),
            Interaction::None => *color = BUTTON_NORMAL_COLOR.into(),
        }
    }
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
