/// Space Invaders!
use bevy::prelude::*;
use rand::seq::SliceRandom;

#[derive(Debug)]
struct Player {
    speed: f32,
}

#[derive(Debug)]
enum GameStatus {
    Ongoing,
    Won,
    Lost,
}

#[derive(Debug)]
struct GameState {
    status: GameStatus,
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
struct Enemy {}

#[derive(Debug)]
struct Fleet {
    speed: f32,
    dir: f32,
    x: f32,
    // don't need state for y
}

#[derive(Debug)]
struct EnemyWeapon {
    cooldown: Timer,
    material_id: usize,
}

#[derive(Debug)]
struct EnemyLaser {
    speed: f32,
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
        .add_system(laser_enemy_hit_detection.system())
        .add_system(player_enemy_hit_detection.system())
        .add_system(player_lasers.system())
        .add_system(enemy_movement.system())
        .add_system(enemy_weapons.system())
        .add_system(enemy_lasers.system())
        .add_system(check_for_win.system())
        .add_system(game_over.system())
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
    let enemy_laser_texture_handle = asset_server.load("../assets/textures/laserRed02.png");
    let font_handle = asset_server.load("../assets/fonts/SourceCodePro-Regular.ttf");

    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default())
        // player
        .spawn(SpriteBundle {
            material: materials.add(player_texture_handle.into()),
            transform: Transform {
                translation: Vec3::new(0.0, -256.0, 0.0),
                rotation: Quat::identity(),
                scale: Vec3::new(0.5, 0.5, 0.5),
            },
            ..Default::default()
        })
        .with(Player { speed: 400.0 })
        // weapon
        .insert_resource(MaterialHandles(vec![
            materials.add(player_laser_texture_handle.into()),
            materials.add(enemy_laser_texture_handle.into()),
        ]))
        .with(Weapon {
            fired: false,
            offset: Vec3::new(0.0, 30.0, 0.0),
            cooldown: Timer::from_seconds(0.5, false),
            material_id: 0,
        })
        .with(EnemyWeapon {
            cooldown: Timer::from_seconds(1., false),
            material_id: 1,
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
        .with(Score { value: 0 })
        // enemy fleet position
        .with(Fleet {
            speed: 1.5,
            dir: 1.,
            x: 0.,
        })
        .with(GameState {
            status: GameStatus::Ongoing,
        });

    // individual enemies
    let x_offset = -350.;
    let y_offset = 50.;
    let scale = 0.5;
    for x in 0..11 {
        for y in 0..5 {
            commands
                .spawn(SpriteBundle {
                    material: materials.add(enemy_texture_handle.clone().into()),
                    transform: Transform {
                        translation: Vec3::new(
                            x_offset + x as f32 * 70.,
                            y_offset + (y as f32 * 70.),
                            0.0,
                        ),
                        rotation: Quat::identity(),
                        scale: Vec3::new(scale, scale, scale),
                    },
                    ..Default::default()
                })
                .with(Enemy {});
        }
    }
}

/// Handles all player input, movement of player sprite, and weapon fired status
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
        // TODO maybe something less abrupt
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

/// Manages weapon cooldown and spawning lasers
fn weapons(
    commands: &mut Commands, /* this must be the fist argument for bevy to recognize this as a system */
    time: Res<Time>,
    materials: ResMut<MaterialHandles>,
    mut query: Query<(&mut Weapon, &Transform)>,
) {
    if let Some((mut weapon, transform)) = query.iter_mut().next() {
        weapon.cooldown.tick(time.delta_seconds());
        if weapon.cooldown.finished() && weapon.fired {
            commands
                .spawn(SpriteBundle {
                    material: materials.0[weapon.material_id].clone(),
                    transform: Transform {
                        translation: weapon.offset + transform.translation,
                        scale: Vec3::new(0.5, 0.5, 0.5),
                        ..Default::default()
                    },

                    ..Default::default()
                })
                .with(PlayerLaser { speed: 500.0 });
            weapon.fired = false;
            weapon.cooldown.reset();
        }
    }
}

/// Moves lasers and despawns if out of bounds
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

/// Moves enemies
fn enemy_movement(
    mut enemy_q: Query<(&mut Enemy, &mut Transform)>,
    mut fleet_q: Query<&mut Fleet>,
) {
    let mut fleet = fleet_q.iter_mut().next().unwrap();
    fleet.x += fleet.speed * fleet.dir;
    let mut moved_down = false;
    if f32::abs(fleet.x) >= 200. {
        fleet.dir *= -1.;
        moved_down = true;
    }

    for (_, mut transform) in enemy_q.iter_mut() {
        transform.translation.x += fleet.dir * fleet.speed;
        if moved_down {
            transform.translation.y -= 10.;
        }
    }
}

/// If all enemies are gone, game ends in a win
fn check_for_win(enemy_q: Query<&Enemy>, mut gamestate_q: Query<&mut GameState>) {
    if enemy_q.iter().count() == 0 {
        gamestate_q.iter_mut().next().unwrap().status = GameStatus::Won;
    }
}

/// Manages weapon cooldown and spawning lasers
fn enemy_weapons(
    commands: &mut Commands, /* this must be the fist argument for bevy to recognize this as a system */
    time: Res<Time>,
    materials: ResMut<MaterialHandles>,
    mut weapon_q: Query<&mut EnemyWeapon>,
    enemy_q: Query<(&Enemy, &Transform)>,
) {
    if let Some(mut weapon) = weapon_q.iter_mut().next() {
        weapon.cooldown.tick(time.delta_seconds());
        if weapon.cooldown.finished() {
            let mut enemies = enemy_q.iter().collect::<Vec<_>>();
            enemies.shuffle(&mut rand::thread_rng());
            if let Some((_enemy, &transform)) = enemies.iter().next() {
                commands
                    .spawn(SpriteBundle {
                        material: materials.0[weapon.material_id].clone(),
                        transform: Transform {
                            translation: transform.translation,
                            scale: Vec3::new(0.5, 0.5, 0.5),
                            ..Default::default()
                        },

                        ..Default::default()
                    })
                    .with(EnemyLaser { speed: 300.0 });
            }
            weapon.cooldown.reset();
        }
    }
}

/// Moves enemy lasers and despawns if out of bounds
fn enemy_lasers(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &EnemyLaser, &mut Transform)>,
) {
    for (ent, laser, mut transform) in query.iter_mut() {
        transform.translation -= Vec3::new(0.0, laser.speed * time.delta_seconds(), 0.0);
        // despawn if laser goes outside window bounds
        if transform.translation.y < -WIN_H / 2. {
            commands.despawn(ent);
        }
    }
}

/// Return if t1 and t2 are within `dist` units of each other in both x and y axes
fn collided(t1: &Vec3, t2: &Vec3, dist: f32) -> bool {
    f32::abs(t1.x - t2.x) <= dist && f32::abs(t1.y - t2.y) <= dist
}

/// Check if any player lasers have hit any enemies, despawn the enemy and laser if so and update
/// score and increase fleet speed
fn laser_enemy_hit_detection(
    commands: &mut Commands,
    mut enemy_q: Query<(Entity, &mut Enemy, &mut Transform)>,
    mut laser_q: Query<(Entity, &mut PlayerLaser, &mut Transform)>,
    mut score_q: Query<(&mut Score, &mut Text)>,
    mut fleet_q: Query<&mut Fleet>,
) {
    let mut fleet = fleet_q.iter_mut().next().unwrap();
    let (mut score, mut text) = score_q.iter_mut().next().unwrap();
    for (enemy_ent, _, enemy_transform) in enemy_q.iter_mut() {
        for (laser_ent, _, laser_transform) in laser_q.iter_mut() {
            if collided(
                &enemy_transform.translation,
                &laser_transform.translation,
                25., // rough estimate of how big the enemies are
            ) {
                commands.despawn(enemy_ent);
                commands.despawn(laser_ent);
                score.value += 1;
                text.value = format!("{}", score.value);
                fleet.speed += 0.2;
            }
        }
    }
}

/// Check if player has hit an enemy or enemy laser, if so: gg no re
fn player_enemy_hit_detection(
    commands: &mut Commands,
    mut enemy_q: Query<(&Enemy, &Transform)>,
    mut player_q: Query<(Entity, &Player, &Transform)>,
    mut laser_q: Query<(Entity, &EnemyLaser, &Transform)>,
    mut gamestate_q: Query<&mut GameState>,
) {
    if let Some((player, _, player_transform)) = player_q.iter_mut().next() {
        let mut gamestate = gamestate_q.iter_mut().next().unwrap();
        for (_, enemy_transform) in enemy_q.iter_mut() {
            if collided(
                &enemy_transform.translation,
                &player_transform.translation,
                35.,
            ) {
                gamestate.status = GameStatus::Lost;
                commands.despawn(player);
            }
        }
        for (laser, _, laser_transform) in laser_q.iter_mut() {
            if collided(
                &laser_transform.translation,
                &player_transform.translation,
                20.,
            ) {
                gamestate.status = GameStatus::Lost;
                commands.despawn(player);
                commands.despawn(laser);
            }
        }
    }
}

/// Check if the game is over by either winning or losing
fn game_over(gamestate_q: Query<&GameState>, mut score_q: Query<(&mut Score, &mut Text)>) {
    // temporarily using score text to communicate game end state
    let (_, mut text) = score_q.iter_mut().next().unwrap();
    match gamestate_q.iter().next().unwrap().status {
        GameStatus::Ongoing => (),
        GameStatus::Won => text.value = "you won!".into(),
        GameStatus::Lost => text.value = "you lost!".into(),
    }
}
