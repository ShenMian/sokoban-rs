use bevy::{color::palettes::css::*, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{components::*, resources::*, Action};

/// Sets up the heads-up display (HUD) on the screen.
pub fn setup_hud(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
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
            TextFont::from_font_size(14.0),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(14.0),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Moves : "),
            TextFont::from_font_size(14.0),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(14.0),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Pushes: "),
            TextFont::from_font_size(14.0),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(14.0),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Best moves : "),
            TextFont::from_font_size(14.0),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(14.0),
            TextColor(GOLD.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("Best pushes: "),
            TextFont::from_font_size(14.0),
            TextColor(SEA_GREEN.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new(""),
            TextFont::from_font_size(14.0),
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
    let hud = hud.single_mut();
    let board = &board.single().board;

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
pub fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button = |parent: &mut ChildBuilder, action, img_path| {
        parent
            .spawn((
                Name::new("Button"),
                action,
                Button,
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BorderColor(Color::NONE),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ImageNode::new(asset_server.load(img_path)),
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Percent(90.0),
                        ..default()
                    },
                ));
            });
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            button(
                parent,
                Action::ToggleInstantMove,
                "textures/instant_move_off.png",
            );
            button(
                parent,
                Action::ToggleAutomaticSolution,
                "textures/automatic_solution.png",
            );
            button(parent, Action::PreviousLevel, "textures/previous.png");
            button(parent, Action::NextLevel, "textures/next.png");
        });
}

/// Updates the state of buttons based on config.
pub fn update_button_state(
    mut buttons: Query<(&Action, &Children), With<Button>>,
    mut image: Query<&mut ImageNode>,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
) {
    if !config.is_changed() {
        return;
    }
    for (button, children) in &mut buttons {
        if *button == Action::ToggleInstantMove {
            let mut image = image.get_mut(children[0]).unwrap();
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
