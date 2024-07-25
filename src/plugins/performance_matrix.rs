use bevy::{
    color::palettes::css::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct PerformanceMatrixPlugin;

impl Plugin for PerformanceMatrixPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

#[derive(Component)]
pub struct PerformanceCounter;

#[derive(Bundle)]
pub struct PerformanceBundle {
    text: TextBundle,
    performance_counter: PerformanceCounter,
}

impl PerformanceBundle {
    pub fn new() -> PerformanceBundle {
        const ALPHA: f32 = 0.8;
        let text_section = move |color, value: &str| {
            TextSection::new(
                value,
                TextStyle {
                    font_size: 14.0,
                    color,
                    ..default()
                },
            )
        };
        PerformanceBundle {
            text: TextBundle::from_sections([
                text_section(AQUA.with_alpha(ALPHA).into(), "FPS     : "),
                text_section(Color::default().with_alpha(ALPHA), ""),
                text_section(AQUA.with_alpha(ALPHA).into(), "FPS(SMA): "),
                text_section(Color::default().with_alpha(ALPHA), ""),
                text_section(AQUA.with_alpha(ALPHA).into(), "FPS(EMA): "),
                text_section(Color::default().with_alpha(ALPHA), ""),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            }),
            performance_counter: PerformanceCounter,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(PerformanceBundle::new());
}

fn update(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<PerformanceCounter>>,
) {
    let mut text = query.single_mut();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            update_fps(raw, &mut text.sections[1]);
        }
        if let Some(sma) = fps.average() {
            update_fps(sma, &mut text.sections[3]);
        }
        if let Some(ema) = fps.smoothed() {
            update_fps(ema, &mut text.sections[5]);
        }
    }
}

fn update_fps(value: f64, section: &mut TextSection) {
    section.value = format!("{value:.2}\n");
    section.style.color = match value {
        v if v < 30.0 => RED.into(),
        v if v < 60.0 => YELLOW.into(),
        _ => LIME.into(),
    };
}
