use bevy::{color::palettes::css::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

/// Sets up the version information text on the screen.
fn setup(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    const FONT_SIZE: FontSize = FontSize::Px(12.0);

    commands.spawn((
        Name::new("Version information"),
        Text::new("version: ".to_string() + env!("CARGO_PKG_VERSION")),
        TextFont::from_font_size(FONT_SIZE),
        TextColor(GRAY.with_alpha(ALPHA).into()),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(5.0),
            right: px(5.0),
            ..default()
        },
    ));
}
