use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct PerformanceMatrixPlugin;

impl Plugin for PerformanceMatrixPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup_performance_matrix)
            .add_systems(Update, update_performance_matrix);
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
                text_section(Color::GREEN.with_a(ALPHA), "FPS     : "),
                text_section(Color::CYAN.with_a(ALPHA), ""),
                text_section(Color::GREEN.with_a(ALPHA), "FPS(SMA): "),
                text_section(Color::CYAN.with_a(ALPHA), ""),
                text_section(Color::GREEN.with_a(ALPHA), "FPS(EMA): "),
                text_section(Color::CYAN.with_a(ALPHA), ""),
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

fn setup_performance_matrix(mut commands: Commands) {
    commands.spawn(PerformanceBundle::new());
}

fn update_performance_matrix(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<PerformanceCounter>>,
) {
    let mut text = query.single_mut();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            text.sections[1].value = format!("{raw:.2}\n");
        }
        if let Some(sma) = fps.average() {
            text.sections[3].value = format!("{sma:.2}\n");
        }
        if let Some(ema) = fps.smoothed() {
            text.sections[5].value = format!("{ema:.2}\n");
        }
    }
}
