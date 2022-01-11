use bevy::{input::gamepad::GamepadButton, prelude::*};

#[derive(Default)]
pub struct PlayerInput {
    pub axis: Vec2,
    pub accel: f32,
    pub brake: f32,
    pub yaw: f32,
    pub camera_axis: Vec2,
}

pub fn gamepad_system(
    gamepads: Res<Gamepads>,
    _button_inputs: Res<Input<GamepadButton>>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut player_input: ResMut<PlayerInput>,
) {
    for gamepad in gamepads.iter().cloned() {
        let right_trigger = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::RightTrigger2))
            .unwrap();

        if right_trigger.abs() > 0.01 {
            player_input.accel = right_trigger;
        } else {
            player_input.accel = 0.0;
        }

        let left_trigger = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::LeftTrigger2))
            .unwrap();

        if left_trigger.abs() > 0.01 {
            player_input.brake = left_trigger;
        } else {
            player_input.brake = 0.0;
        }

        let left_stick_x = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        let left_stick_y = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))
            .unwrap();

        if left_stick_x.abs() > 0.01 || left_stick_y.abs() > 0.01 {
            player_input.axis = Vec2::new(left_stick_x, left_stick_y);
        } else {
            player_input.axis = Vec2::ZERO;
        }

        let left_shoulder = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::LeftTrigger))
            .unwrap();
        let right_shoulder = button_axes
            .get(GamepadButton(gamepad, GamepadButtonType::RightTrigger))
            .unwrap();

        let right_stick_x = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::RightStickX))
            .unwrap();
        let right_stick_y = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::RightStickY))
            .unwrap();

        if right_stick_x.abs() > 0.1 || right_stick_y.abs() > 0.1 {
            player_input.camera_axis = Vec2::new(right_stick_x, right_stick_y);
        } else {
            player_input.camera_axis = Vec2::ZERO;
        }

        player_input.yaw = left_shoulder - right_shoulder;
    }
}
