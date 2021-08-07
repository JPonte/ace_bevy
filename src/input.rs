use bevy::{prelude::*, utils::HashSet};

#[derive(Default)]
pub struct PlayerInput {
    pub axis: Vec2,
    pub accel: f32,
    pub brake: f32,
    pub yaw: f32,
}

#[derive(Default)]
pub struct GamepadLobby {
    pub gamepads: HashSet<Gamepad>,
}

pub fn connection_system(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                lobby.gamepads.insert(*gamepad);
                println!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                lobby.gamepads.remove(gamepad);
                println!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}

pub fn gamepad_system(
    lobby: Res<GamepadLobby>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut player_input: ResMut<PlayerInput>,
) {
    for gamepad in lobby.gamepads.iter().cloned() {
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

        player_input.yaw = left_shoulder - right_shoulder;
    }
}
