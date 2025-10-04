#![allow(clippy::type_complexity)]

use bevy::{color::palettes::css::*, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{components::*, resources::*, Action};
use crate::{state::*, systems::input::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (setup_hud, setup_buttons));
    app.add_systems(
        Update,
        (
            button_visual_effect,
            update_button_state,
            (button_input_to_action, handle_actions).chain(),
        ),
    );
    app.add_systems(Update, update_hud.run_if(in_state(AppState::Main)));
}

/// Sets up the heads-up display (HUD) on the screen.
pub fn setup_hud(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    const FONT_SIZE: f32 = 14.0;

    commands
        .spawn((
            Name::new("HUD"),
            Hud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("Level : "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Moves : "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Pushes: "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Best moves : "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Best pushes: "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ));
}

/// Updates the heads-up display (HUD).
pub fn update_hud(
    mut hud: Query<Entity, With<Hud>>,
    mut writer: TextUiWriter,
    board: Query<&Board>,
    level_id: Res<LevelId>,
    database: Res<Database>,
) {
    let hud = hud.single_mut().unwrap();
    let board = &board.single().unwrap().board;

    if level_id.is_changed() {
        *writer.text(hud, 1) = format!("#{}\n", level_id.0);

        let database = database.lock().unwrap();
        *writer.text(hud, 7) = format!(
            "{}\n",
            database
                .best_move_solution(level_id.0)
                .unwrap_or_default()
                .moves()
        );
        *writer.text(hud, 9) = format!(
            "{}\n",
            database
                .best_push_solution(level_id.0)
                .unwrap_or_default()
                .pushes()
        );
    }

    *writer.text(hud, 3) = format!("{}\n", board.actions().moves());
    *writer.text(hud, 5) = format!("{}\n", board.actions().pushes());
}

/// Sets up buttons on the screen.
pub fn setup_buttons(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Buttons"),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        children![
            icon_button(
                Action::ToggleInstantMove,
                asset_server.load("textures/instant_move_off.png"),
            ),
            icon_button(
                Action::ToggleAutomaticSolution,
                asset_server.load("textures/automatic_solution.png"),
            ),
            icon_button(
                Action::PreviousLevel,
                asset_server.load("textures/previous.png"),
            ),
            icon_button(Action::NextLevel, asset_server.load("textures/next.png")),
        ],
    ));
}

fn icon_button(action: Action, texture: Handle<Image>) -> impl Bundle {
    const ICON_BUTTON_SIZE: f32 = 64.0;
    (
        Name::new("Icon Button"),
        action,
        Button,
        Node {
            width: Val::Px(ICON_BUTTON_SIZE),
            height: Val::Px(ICON_BUTTON_SIZE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BorderColor::all(Color::NONE),
        ImageNode::new(texture),
    )
}

/// Updates the state of buttons based on config.
pub fn update_button_state(
    mut buttons: Query<(&Action, &mut ImageNode), With<Button>>,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
) {
    if !config.is_changed() {
        return;
    }
    for (button, mut image) in &mut buttons {
        if *button == Action::ToggleInstantMove {
            image.image = if config.instant_move {
                asset_server.load("textures/instant_move_on.png")
            } else {
                asset_server.load("textures/instant_move_off.png")
            };
        }
    }
}

/// Applies visual effects to buttons based on user interaction.
pub fn button_visual_effect(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    const BUTTON_NORMAL_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 0.8);
    const BUTTON_HOVERED_COLOR: Color = Color::srgba(0.55, 0.55, 0.55, 0.8);
    const BUTTON_PRESSED_COLOR: Color = Color::srgba(0.35, 0.75, 0.35, 0.8);

    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::Pressed => BUTTON_PRESSED_COLOR,
            Interaction::Hovered => BUTTON_HOVERED_COLOR,
            Interaction::None => BUTTON_NORMAL_COLOR,
        }
        .into();
    }
}

/// Converts button interactions to input actions.
pub fn button_input_to_action(
    buttons: Query<(&Interaction, &Action), (Changed<Interaction>, With<Button>)>,
    mut action_state: ResMut<ActionState<Action>>,
) {
    for (interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            action_state.press(action);
        }
    }
}
