use bevy::prelude::*;

#[derive(Component)]
pub struct HelpText;

pub fn show_helptext(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q: Query<Entity, With<HelpText>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        let entity = q.get_single_mut();
        if let Ok(e) = entity {
            commands.entity(e).despawn();
        } else {
            commands.spawn((
                    Text::new("Help:\n\nr: toggle animated camera\n\
                    a: increase update speed\n\
                    s: decrease update speed\n\
                    n: spawn new cubes\n\
                    m: spawn new cubes in specified center area\n\
                    h: toggle help text\n\
                    press 'space' to pause the system\n\n\
                    press 'q' or 'esc' to quit"),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(12.0),
                        left: Val::Px(12.0),
                        ..default()
                    },
                    HelpText,
            ));
        }
    }
}
