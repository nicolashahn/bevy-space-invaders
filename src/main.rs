/// Space Invaders!
use bevy::prelude::*;

#[derive(Debug)]
struct Player {
    speed: f32,
}

#[derive(Debug)]
struct Weapon {
    fired: bool,
    offset: Vec3,
    cooldown: Timer,
    material_id: usize,
}

#[derive(Debug)]
struct PlayerLaser {
    speed: f32,
}

#[derive(Debug)]
struct MaterialHandles(Vec<Handle<ColorMaterial>>);

#[derive(Debug)]
struct Enemy {
    speed: f32,
    dir: f32,
}

#[derive(Debug)]
struct Score {
    value: i32,
}

const WIN_H: f32 = 1024.;
const WIN_W: f32 = 1024.;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(WindowDescriptor {
            title: "Space Invaders".to_string(),
            width: WIN_W,
            height: WIN_H,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_startup_system(setup.system())
        .add_system(player_control.system())
        .add_system(weapons.system())
        .add_system(enemy_hit_detection.system())
        .add_system(player_lasers.system())
        .add_system(enemies.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_texture_handle = asset_server.load("../assets/textures/playerShip1_blue.png");
    let player_laser_texture_handle = asset_server.load("../assets/textures/laserBlue01.png");
    let enemy_texture_handle = asset_server.load("../assets/textures/enemyRed1.png");
    let font_handle = asset_server.load("../assets/fonts/SourceCodePro-Regular.ttf");

    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // player
        .spawn(SpriteBundle {
            material: materials.add(player_texture_handle.into()),
            transform: Transform::from_translation(Vec3::new(0.0, -256.0, 0.0)),
            ..Default::default()
        })
        .with(Player { speed: 400.0 })
        // weapon
        .insert_resource(MaterialHandles(vec![
            materials.add(player_laser_texture_handle.into())
        ]))
        .with(Weapon {
            fired: false,
            offset: Vec3::new(0.0, 30.0, 0.0),
            cooldown: Timer::from_seconds(0.4, false),
            material_id: 0,
        })
        // score text
        .spawn(TextBundle {
            text: Text {
                value: "0".into(),
                font: font_handle,
                style: TextStyle {
                    color: Color::WHITE,
                    font_size: 60.,
                    ..Default::default()
                },
            },
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Score { value: 0 });

    for i in 0..5 {
        commands
            .spawn(SpriteBundle {
                material: materials.add(enemy_texture_handle.clone().into()),
                transform: Transform::from_translation(Vec3::new(i as f32 * 100., 256.0, 0.0)),
                ..Default::default()
            })
            .with(Enemy { speed: 2., dir: 1. });
    }
}

/// handles all player input
fn player_control(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform, Option<&mut Weapon>)>,
) {
    let mut movement = 0.0;
    let mut weapon_fired = false;
    if keyboard_input.pressed(KeyCode::Left) {
        movement -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        movement += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Space) {
        weapon_fired = true;
    }
    if keyboard_input.pressed(KeyCode::Escape) {
        panic!("player terminated game");
    }

    for (player, mut transform, weapon) in query.iter_mut() {
        transform.translation +=
            Vec3::new(movement * player.speed * time.delta_seconds(), 0.0, 0.0);
        if let Some(mut w) = weapon {
            w.fired = weapon_fired;
        }
    }
}

/// manages weapon cooldown and spawning lasers
fn weapons(
    commands: &mut Commands, /* this must be the fist argument for bevy to recognize this as a system */
    time: Res<Time>,
    materials: ResMut<MaterialHandles>,
    mut query: Query<(&mut Weapon, &Transform)>,
) {
    for (mut weapon, transform) in query.iter_mut() {
        weapon.cooldown.tick(time.delta_seconds());
        if weapon.cooldown.finished() && weapon.fired {
            commands
                .spawn(SpriteBundle {
                    material: materials.0[weapon.material_id].clone(),
                    transform: Transform::from_translation(weapon.offset + transform.translation),
                    ..Default::default()
                })
                .with(PlayerLaser { speed: 1000.0 });
            weapon.fired = false;
            weapon.cooldown.reset();
        }
    }
}

/// moves lasers and despawns if out of bounds
fn player_lasers(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &PlayerLaser, &mut Transform)>,
) {
    for (ent, laser, mut transform) in query.iter_mut() {
        transform.translation += Vec3::new(0.0, laser.speed * time.delta_seconds(), 0.0);
        // despawn if laser goes outside window bounds
        if transform.translation.y > WIN_H / 2. {
            commands.despawn(ent);
        }
    }
}

/// moves enemies
fn enemies(mut query: Query<(&mut Enemy, &mut Transform)>) {
    for (mut enemy, mut transform) in query.iter_mut() {
        transform.translation += Vec3::new(enemy.dir * enemy.speed, 0., 0.);
        if f32::abs(transform.translation.x) == 480. {
            enemy.dir *= -1.;
            transform.translation += Vec3::new(0., -5., 0.);
        }
    }
}

/// Return if t1 and t2 are within `dist` units of each other in both x and y axes
fn collided(t1: &Vec3, t2: &Vec3, dist: f32) -> bool {
    f32::abs(t1.x - t2.x) <= dist && f32::abs(t1.y - t2.y) <= dist
}

/// Check if any player lasers have hit any enemies
fn enemy_hit_detection(
    commands: &mut Commands,
    mut enemy_q: Query<(Entity, &mut Enemy, &mut Transform)>,
    mut laser_q: Query<(Entity, &mut PlayerLaser, &mut Transform)>,
    mut score_q: Query<(&mut Score, &mut Text)>,
) {
    //let mut player = p_query.iter_mut().next().unwrap();
    let (mut score, mut text) = score_q.iter_mut().next().unwrap();
    for (enemy_ent, _, enemy_transform) in enemy_q.iter_mut() {
        for (laser_ent, _, laser_transform) in laser_q.iter_mut() {
            if collided(
                &enemy_transform.translation,
                &laser_transform.translation,
                60., // rough estimate of how big the enemies are
            ) {
                commands.despawn(enemy_ent);
                commands.despawn(laser_ent);
                score.value += 1;
                text.value = format!("{}", score.value);
            }
        }
    }
}
