use crate::systems::{input::*, render::*};

use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (adjust_viewport, adjust_camera_scale));
        app.add_systems(FixedUpdate, smooth_camera_motion);
    }
}
