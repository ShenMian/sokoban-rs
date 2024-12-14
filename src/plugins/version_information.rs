use bevy::{color::palettes::css::*, prelude::*};

pub struct VersionInformationPlugin;

impl Plugin for VersionInformationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

/// Sets up the version information text on the screen.
fn setup(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    commands.spawn((
        Name::new("Version information"),
        Text::new("version: ".to_string() + env!("CARGO_PKG_VERSION")),
        TextFont::from_font_size(14.0),
        TextColor(GRAY.with_alpha(ALPHA).into()),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
    ));
}
