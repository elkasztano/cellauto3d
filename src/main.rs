use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
    window::WindowMode,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use cellauto3d::{
    cli::{Cli, ColorGradient, LightMode},
    cube_density,
    gradient::{adjustable_bw, adjustable_spectrum, petrol},
    helptext::show_helptext,
    rules::Rules,
    system::{AutoSystem3d, SystemDims},
    update::{
        adjust_timer, manage_panorbit, quit, spawn_new_at_keystroke, spawn_pseudorandom_full,
        update_system,
    },
    GlobalData, GlobalStatic, SystemTimer,
};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    let dims = SystemDims::new_cube_clamped(16, 96, cli.edge_length);
    let grad = match cli.color_gradient {
        ColorGradient::Rainbow => adjustable_spectrum(0.2, 0.8),
        ColorGradient::BlackWhite => adjustable_bw(0.1, 0.9),
        ColorGradient::Petrol => petrol(1.0),
    };
    let auto_system = AutoSystem3d::new_from_dims(&dims);
    let rules = Rules::parse_from_str(&cli.rules).expect("unable to parse rules correctly");
    eprintln!("Rules:\n{}", &rules);
    let min = cube_density(cli.edge_length, cli.minimum);
    let max = cube_density(cli.edge_length, cli.maximum);
    // manage plugins and fullscreen mode
    let plugins = if cli.fullscreen {
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resizable: false,
                mode: WindowMode::Fullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        })
    } else {
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CellAuto3D".to_string(),
                ..default()
            }),
            ..default()
        })
    };

    let mut app = App::new();

    match &cli.light_mode {
        LightMode::Bloom => {
            app.insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)));
        }
        _ => {}
    }

    app.add_plugins(plugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_system,
                spawn_new_at_keystroke,
                adjust_timer,
                show_helptext,
                manage_panorbit,
                quit,
            ),
        )
        .insert_resource(auto_system)
        .insert_resource(SystemTimer::millis(125))
        .insert_resource(GlobalData::new(cli.seed))
        .insert_resource(GlobalStatic::new(grad, dims, min, max))
        .insert_resource(rules)
        .insert_resource(cli);

    app.run();
}

fn setup(
    mut commands: Commands,
    para_commands: ParallelCommands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    glstat: Res<GlobalStatic>,
    auto_system: ResMut<AutoSystem3d>,
    global_data: ResMut<GlobalData>,
    mut ambient_light: ResMut<AmbientLight>,
    rules: Res<Rules>,
    cli: Res<Cli>,
) {
    // light, dependent on settings
    let (illuminance, ambi) = match cli.light_mode {
        LightMode::Bloom => (1000.0, 250.0),
        LightMode::Normal => (2000.0, 500.0),
    };
    commands.spawn((
        DirectionalLight {
            illuminance,
            ..default()
        },
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    ambient_light.brightness = ambi;

    // (panorbit)camera
    match cli.light_mode {
        LightMode::Bloom => {
            commands.spawn((
                Transform::from_translation(Vec3::new(0.0, 1.5, 15.0)),
                PanOrbitCamera::default(),
                Camera {
                    hdr: true,
                    ..default()
                },
                Tonemapping::TonyMcMapface,
                Bloom::NATURAL,
            ));
        }
        LightMode::Normal => {
            commands.spawn((
                Transform::from_translation(Vec3::new(10.0, 2.5, 10.0)),
                PanOrbitCamera::default(),
            ));
        }
    }

    // initial fill
    spawn_pseudorandom_full(
        para_commands,
        meshes,
        materials,
        auto_system,
        global_data,
        rules,
        glstat,
        cli,
    );
}
