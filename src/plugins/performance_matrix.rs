use bevy::{
    color::palettes::css::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update);
}

#[derive(Component)]
pub struct PerformanceMatrix;

fn setup(mut commands: Commands) {
    const ALPHA: f32 = 0.8;
    const FONT_SIZE: f32 = 12.0;

    commands
        .spawn((
            Name::new("Performance matrix"),
            PerformanceMatrix,
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::new("FPS     : "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(AQUA.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("\n"),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(Color::default().with_alpha(ALPHA)),
        ))
        .with_child((
            TextSpan::new("FPS(SMA): "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(AQUA.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("\n"),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(Color::default().with_alpha(ALPHA)),
        ))
        .with_child((
            TextSpan::new("FPS(EMA): "),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(AQUA.with_alpha(ALPHA).into()),
        ))
        .with_child((
            TextSpan::new("\n"),
            TextFont::from_font_size(FONT_SIZE),
            TextColor(Color::default().with_alpha(ALPHA)),
        ));
}

fn update(
    diagnostics: Res<DiagnosticsStore>,
    query: Query<Entity, With<PerformanceMatrix>>,
    mut writer: TextUiWriter,
) {
    let text = query.single().unwrap();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            update_text_span(2, raw, text, &mut writer);
        }
        if let Some(sma) = fps.average() {
            update_text_span(4, sma, text, &mut writer);
        }
        if let Some(ema) = fps.smoothed() {
            update_text_span(6, ema, text, &mut writer);
        }
    }
}

fn update_text_span(index: usize, value: f64, text: Entity, writer: &mut TextUiWriter) {
    *writer.text(text, index) = format!("{value:.2}\n");
    *writer.color(text, index) = match value {
        v if v < 30.0 => RED.into(),
        v if v < 60.0 => YELLOW.into(),
        _ => LIME.into(),
    };
}
