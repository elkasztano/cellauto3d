use crate::{
    calc_spawn_coords,
    cli::{Cli, LightMode},
    rel_density,
    rules::{Neighbourhood, Rules},
    system::{AutoSystem3d, Automaton, SysChange},
    GlobalData, GlobalStatic, SystemTimer, ALPHA, BLOOM, CUBE_SIZE,
};
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use colorgrad::Gradient;
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use xorwowgen::xorwow64::XorA;

pub fn update_system(
    par_com: ParallelCommands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sys3d: ResMut<AutoSystem3d>,
    mut config: ResMut<SystemTimer>,
    time: Res<Time>,
    rules: Res<Rules>,
    mut global_data: ResMut<GlobalData>,
    global_stat: Res<GlobalStatic>,
    cli: Res<Cli>,
) {
    config.timer.tick(time.delta());
    if config.timer.finished() && !config.stopped {
        // get color of current generation
        let c = global_stat
            .gradient()
            .reflect_at((global_data.generation() as f32) / 20.0);
        let mesh_handle = meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE));
        // set emission if bloom mode is chosen
        let mat_handle = match cli.light_mode {
            LightMode::Bloom => materials.add(StandardMaterial {
                emissive: LinearRgba::new(c.r * BLOOM, c.g * BLOOM, c.b * BLOOM, ALPHA),
                alpha_mode: AlphaMode::Add,
                ..default()
            }),
            LightMode::Normal => materials.add(Color::srgb(c.r, c.g, c.b)),
        };
        // thread safe smart pointer for the changes that define
        // the new system state, preallocate memory to optimize speed
        let changes = Arc::new(Mutex::new(Vec::<SysChange>::with_capacity(
            global_stat.max_amount,
        )));
        // keep track of the number of living cubes
        let am_counter = Arc::new(Mutex::new(0isize));
        // concurrently iterate over the system
        global_stat
            .dims()
            .range_x()
            .into_par_iter()
            .for_each(|i: usize| {
                // thread local changes, merged after each thread has finished
                // preallocate space to optimize speed
                let mut thread_local_changes =
                    Vec::<SysChange>::with_capacity(global_stat.max_amount_per_thread);
                // keep track of spawned/despawned cubes per thread
                // note that within a single thread 'despawned' can be higher
                // than 'spawned'
                let mut spawned = 0isize;
                let mut despawned = 0isize;
                // thread local iteration
                for j in global_stat.dims().range_y() {
                    for k in global_stat.dims().range_z() {
                        let uxyz = (i, j, k); // the 'u' stands for 'unsigned'
                                              // count neighbours and apply rules
                        let n = match rules.neighbourhood() {
                            Neighbourhood::Moore => {
                                sys3d.count_neighbours_moore(uxyz, &global_stat.dims)
                            }
                            Neighbourhood::VonNeumann => {
                                sys3d.count_neighbours_von_neumann(uxyz, &global_stat.dims)
                            }
                        };
                        if rules.check_despawn(n) {
                            if let Some(at) = sys3d.get_at_xyz(uxyz) {
                                if at.life() <= 0 {
                                    // despawn if life is already at zero
                                    thread_local_changes.push(SysChange::empty(i, j, k));
                                    par_com.command_scope(|mut commands| {
                                        commands.entity(at.entity()).despawn();
                                    });
                                    despawned += 1;
                                } else if at.life() > 0 {
                                    // if life is larger than zero, reduce it by one
                                    thread_local_changes
                                        .push(SysChange::change_life(i, j, k, at, -1));
                                    // shrink cube in order to visualize aging
                                    par_com.command_scope(|mut commands| {
                                        commands
                                            .entity(at.entity())
                                            .entry::<Transform>()
                                            .and_modify(|mut t| {
                                                t.scale *= 0.75;
                                            });
                                    });
                                }
                            }
                        } else if rules.check_spawn(n)
                            && global_data.growth()
                            && sys3d.get_at_xyz(uxyz).is_none()
                        {
                            // spawn cube if spot is empty and neighbour count
                            // is within specified range
                            let sc = calc_spawn_coords(uxyz, &global_stat.dims());
                            let id = par_com.command_scope(|mut commands| {
                                commands
                                    .spawn((
                                        Mesh3d(mesh_handle.clone()),
                                        MeshMaterial3d(mat_handle.clone()),
                                        Transform::from_xyz(sc.0, sc.1, sc.2),
                                    ))
                                    .id()
                            });
                            spawned += 1;
                            thread_local_changes.push(SysChange::spawn(
                                i,
                                j,
                                k,
                                Automaton::new(id, rules.life()),
                            ));
                        }
                    }
                }
                // merge changes
                let mut chg = changes.lock().unwrap();
                chg.append(&mut thread_local_changes);
                // balance of spawned and despawned
                let mut cnt = am_counter.lock().unwrap();
                *cnt += spawned - despawned;
            });
        let all_chg = changes.lock().unwrap();
        // apply all changes to the system to finally create the new state
        sys3d.apply_changes(&*all_chg);
        let cnt = am_counter.lock().unwrap();
        // keep track of currently living cubes
        global_data.increase(*cnt);
        eprint!(
            "amount: {:012}, density: {:4.3}\r",
            global_data.amount(),
            rel_density(global_stat.dims.x(), global_data.amount())
        );
        // avoid general overpopulation and sparseness
        if global_data.amount() > global_stat.maximum() {
            global_data.unset_growth();
        } else if global_data.amount() < global_stat.minimum() {
            global_data.set_growth();
        }
        // keep track of generations
        global_data.advance_gen();
    }
}

pub fn spawn_pseudorandom_core(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sys3d: ResMut<AutoSystem3d>,
    mut global_data: ResMut<GlobalData>,
    rules: Res<Rules>,
    glstat: Res<GlobalStatic>,
    cli: Res<Cli>,
) {
    let c = glstat
        .gradient()
        .reflect_at((global_data.generation() as f32) / 20.0);
    let mesh_handle = meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE));
    let mat_handle = match cli.light_mode {
        LightMode::Bloom => materials.add(StandardMaterial {
            emissive: LinearRgba::new(c.r * BLOOM, c.g * BLOOM, c.b * BLOOM, ALPHA),
            alpha_mode: AlphaMode::Add,
            ..default()
        }),
        LightMode::Normal => materials.add(Color::srgb(c.r, c.g, c.b)),
    };
    let mut changes = Vec::<SysChange>::new();
    let mut count = 0isize;
    let mut rng = XorA::seed_from_u64(global_data.seed());
    for i in glstat.dims().core_range_x(cli.fraction) {
        for j in glstat.dims().core_range_y(cli.fraction) {
            for k in glstat.dims().core_range_z(cli.fraction) {
                if rng.gen_bool(cli.core_density) && sys3d.get_at_xyz((i, j, k)).is_none() {
                    count += 1;
                    let coords = calc_spawn_coords((i, j, k), &glstat.dims());
                    let id = commands
                        .spawn((
                            Mesh3d(mesh_handle.clone()),
                            MeshMaterial3d(mat_handle.clone()),
                            Transform::from_xyz(coords.0, coords.1, coords.2),
                        ))
                        .id();
                    changes.push(SysChange::spawn(i, j, k, Automaton::new(id, rules.life())));
                }
            }
        }
    }
    sys3d.apply_changes(&changes);
    global_data.increase(count);
    global_data.set_seed(rng.next_u64());
}

pub fn spawn_pseudorandom_full(
    par_com: ParallelCommands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sys3d: ResMut<AutoSystem3d>,
    mut global_data: ResMut<GlobalData>,
    rules: Res<Rules>,
    glstat: Res<GlobalStatic>,
    cli: Res<Cli>,
) {
    let c = glstat
        .gradient()
        .reflect_at((global_data.generation() as f32) / 20.0);
    let mesh_handle = meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE));
    let mat_handle = match cli.light_mode {
        LightMode::Bloom => materials.add(StandardMaterial {
            emissive: LinearRgba::new(c.r * BLOOM, c.g * BLOOM, c.b * BLOOM, ALPHA),
            alpha_mode: AlphaMode::Add,
            ..default()
        }),
        LightMode::Normal => materials.add(Color::srgb(c.r, c.g, c.b)),
    };
    let changes = Arc::new(Mutex::new(Vec::<SysChange>::new()));
    let am_count = Arc::new(Mutex::new(0isize));

    glstat
        .dims()
        .range_x()
        .into_par_iter()
        .for_each(|i: usize| {
            let mut local_count = 0;
            let mut local_changes = Vec::<SysChange>::new();
            let mut rng = XorA::seed_from_u64(
                global_data
                    .seed()
                    .wrapping_add(i.wrapping_mul(999999999) as u64),
            );
            for j in glstat.dims().range_y() {
                for k in glstat.dims().range_z() {
                    if rng.gen_bool(cli.density) && sys3d.get_at_xyz((i, j, k)).is_none() {
                        local_count += 1;
                        let coords = calc_spawn_coords((i, j, k), &glstat.dims());
                        let id = par_com.command_scope(|mut commands| {
                            commands
                                .spawn((
                                    Mesh3d(mesh_handle.clone()),
                                    MeshMaterial3d(mat_handle.clone()),
                                    Transform::from_xyz(coords.0, coords.1, coords.2),
                                ))
                                .id()
                        });
                        local_changes.push(SysChange::spawn(
                            i,
                            j,
                            k,
                            Automaton::new(id, rules.life()),
                        ));
                    }
                }
            }
            let mut chg = changes.lock().unwrap();
            chg.append(&mut local_changes);
            let mut cnt = am_count.lock().unwrap();
            *cnt += local_count;
        });

    let chg = changes.lock().unwrap();
    sys3d.apply_changes(&*chg);
    let cnt = am_count.lock().unwrap();
    global_data.increase(*cnt);

    let mut rng = XorA::seed_from_u64(global_data.seed());
    for _ in 0..64 {
        rng.next_u64();
    }
    global_data.set_seed(rng.next_u64());
}

// spawn (pseudo)random cubes at keystroke
pub fn spawn_new_at_keystroke(
    par_com: ParallelCommands,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    sys3d: ResMut<AutoSystem3d>,
    global_data: ResMut<GlobalData>,
    rules: Res<Rules>,
    glstat: Res<GlobalStatic>,
    cli: Res<Cli>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        spawn_pseudorandom_full(
            par_com,
            meshes,
            materials,
            sys3d,
            global_data,
            rules,
            glstat,
            cli,
        );
    } else if keyboard.just_pressed(KeyCode::KeyM) {
        spawn_pseudorandom_core(
            commands,
            meshes,
            materials,
            sys3d,
            global_data,
            rules,
            glstat,
            cli,
        );
    }
}

pub fn adjust_timer(keyboard: Res<ButtonInput<KeyCode>>, mut sys_timer: ResMut<SystemTimer>) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        sys_timer.increase_micros(31250);
    } else if keyboard.just_pressed(KeyCode::KeyA) {
        sys_timer.decrease_micros(31250);
    } else if keyboard.just_pressed(KeyCode::Space) {
        sys_timer.toggle_timer();
    }
}

pub fn manage_panorbit(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for mut pan_orbit in pan_orbit_query.iter_mut() {
            if pan_orbit.enabled {
                pan_orbit.enabled = false;
                pan_orbit.orbit_smoothness = 0.0;
            } else {
                pan_orbit.enabled = true;
                pan_orbit.orbit_smoothness = 0.1;
            }
        }
    }
    for mut pan_orbit in pan_orbit_query.iter_mut() {
        if !pan_orbit.enabled {
            pan_orbit.target_yaw += 15f32.to_radians() * time.delta_secs();
            pan_orbit.force_update = true;
        }
    }
}

pub fn quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
        eprintln!("\n'Q' or 'esc' was pressed - exiting");
        app_exit.send(AppExit::Success);
    }
}
