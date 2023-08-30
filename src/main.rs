use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const GAMEAREA_PADDING: f32 = 80.0;

const PLAYER_SPEED: f32 = 2000.0;
const PLAYER_SIZE: f32 = 90.0;

const ENEMY_PER_ROW: usize = 4;
const ENEMY_SPEED: f32 = 1000.0;
const ENEMY_SIZE: f32 = 102.0;
const ENEMY_SPACING: f32 = 120.0;

const BULLET_SPEED: f32 = 1000.0;
const BULLET_SIZE: f32 = 9.0;

const ENEMY_SPAWN_TIME_SEC: f32 = 2.0;

#[derive(SystemSet, States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum AppState {
    #[default]
    MainMenu,
    Game,
    GameOver,
}

#[derive(SystemSet, States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum SimulationState {
    Running,
    #[default]
    Paused,
}

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct Bullet {}

#[derive(Component, Debug)]
struct Enemy {
    pub direction: Vec2,
    pub row: usize,
    pub col: usize,
}

#[derive(Component)]
struct BounceSound;

#[derive(Component)]
struct LoseSound;

#[derive(Component)]
struct BulletSpawnSound;

#[derive(Component)]
struct BulletHitSound;

#[derive(Default, Resource)]
struct Score {
    pub value: u32,
}

#[derive(Event)]
struct GameOver {
    pub score: u32,
}

#[derive(Resource)]
struct EnemySpawnTimer {
    pub timer: Timer,
}

impl Default for EnemySpawnTimer {
    fn default() -> EnemySpawnTimer {
        EnemySpawnTimer {
            timer: Timer::from_seconds(ENEMY_SPAWN_TIME_SEC, TimerMode::Repeating),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<AppState>()
        .add_state::<SimulationState>()
        .add_event::<GameOver>()
        .init_resource::<Score>()
        .init_resource::<EnemySpawnTimer>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(OnEnter(AppState::Game), (spawn_player, spawn_enemies))
        .add_systems(OnExit(AppState::Game), despawn_entities)
        .add_systems(Update, (toggle_appstate))
        .add_systems(Update, (toggle_simulation).run_if(in_state(AppState::Game)))
        .add_systems(
            Update,
            (
                player_movement,
                player_bounds.after(player_movement),
                enemy_movement,
                enemy_bounds.after(enemy_movement),
                enemy_direction.after(enemy_bounds),
                enemy_hit_player,
                // enemy_spawn_cycle,
                bullet_spawn,
                bullet_movement,
                bullet_bounds,
                bullet_hit_enemy,
                update_score,
                handle_game_over,
            )
                .run_if(in_state(AppState::Game))
                .run_if(in_state(SimulationState::Running)),
        )
        .run()
}

fn toggle_appstate(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    app_state: Res<State<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::G) {
        commands.insert_resource(NextState(Some(SimulationState::Paused)));

        if app_state.get() == &AppState::MainMenu {
            dbg!("SimulationState::Game");
            commands.insert_resource(NextState(Some(AppState::Game)));
        }
        if app_state.get() == &AppState::Game {
            dbg!("SimulationState::MainMenu");
            commands.insert_resource(NextState(Some(AppState::MainMenu)));
        }
    }
}

fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..Default::default()
    });
}

fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            texture: asset_server.load("png/ufoGreen.png"),
            ..Default::default()
        },
        Player {},
    ));
}

fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    for i in 0..ENEMY_PER_ROW {
        let x = GAMEAREA_PADDING + ENEMY_SPACING * i as f32;
        let y = window.height() - GAMEAREA_PADDING;

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: asset_server.load("png/Enemies/enemyRed3.png"),
                ..Default::default()
            },
            Enemy {
                direction: Vec2::new(1.0, 0.0).normalize(),
                row: 0,
                col: i,
            },
        ));
    }
}

fn despawn_entities(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    dbg!("despawn_entities");

    if let Ok(player_entity) = player_query.get_single() {
        commands.entity(player_entity).despawn();
    }

    for enemy_entity in enemy_query.iter() {
        commands.entity(enemy_entity).despawn();
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Right) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Up) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::Down) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        player_transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

fn player_bounds(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let window = window_query.get_single().unwrap();
    let half_player_size = PLAYER_SIZE / 2.0;

    let x_min = half_player_size;
    let x_max = window.width() - half_player_size;
    let y_min = half_player_size;
    let y_max = window.height() - half_player_size;

    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let mut translation = player_transform.translation;

        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }

        if translation.y < y_min {
            translation.y = y_min;
        } else if translation.y > y_max {
            translation.y = y_max;
        }

        player_transform.translation = translation;
    }
}

fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    dbg!("runninig");
    for (mut enemy_transform, enemy) in enemy_query.iter_mut() {
        let direction = Vec3::new(enemy.direction.x, enemy.direction.y, 0.0);
        enemy_transform.translation += direction * ENEMY_SPEED * time.delta_seconds();
    }
}

fn enemy_bounds(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
) {
    let window = window_query.get_single().unwrap();
    let half_enemy_size = ENEMY_SIZE / 2.0;

    for (mut enemy_transform, enemy) in enemy_query.iter_mut() {
        let x_min = GAMEAREA_PADDING + (ENEMY_SPACING * enemy.col as f32) + half_enemy_size;
        let x_max = window.width()
            - GAMEAREA_PADDING
            - (ENEMY_SPACING * (ENEMY_PER_ROW - enemy.col - 1) as f32)
            - half_enemy_size;

        let mut translation = enemy_transform.translation;

        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }

        enemy_transform.translation = translation;
    }
}

fn enemy_direction(
    mut _commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    _asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();
    let half_enemy_size = ENEMY_SIZE / 2.0;

    for (enemy_transform, mut enemy) in enemy_query.iter_mut() {
        let x_min = GAMEAREA_PADDING + (ENEMY_SPACING * enemy.col as f32) + half_enemy_size;
        let x_max = window.width()
            - GAMEAREA_PADDING
            - (ENEMY_SPACING * (ENEMY_PER_ROW - enemy.col - 1) as f32)
            - half_enemy_size;

        let translation = enemy_transform.translation;
        let mut direction_changed = false;

        if translation.x == x_min || translation.x == x_max {
            enemy.direction.x *= -1.0;
            direction_changed = true;
        }

        if direction_changed {
            // commands.spawn((
            //     AudioBundle {
            //         source: asset_server.load("audio/sfx_zap.ogg"),
            //         settings: PlaybackSettings::ONCE,
            //     },
            //     BounceSound,
            // ));
        }
    }
}

fn enemy_hit_player(
    mut commands: Commands,
    mut game_over_event_writer: EventWriter<GameOver>,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single_mut() {
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let distance = player_transform
                .translation
                .distance(enemy_transform.translation);
            let player_radius = PLAYER_SIZE / 2.0;
            let enemy_radius = ENEMY_SIZE / 2.0;

            if distance < player_radius + enemy_radius {
                commands.spawn((
                    AudioBundle {
                        source: asset_server.load("audio/sfx_lose.ogg"),
                        settings: PlaybackSettings::ONCE,
                    },
                    LoseSound,
                ));

                commands.entity(enemy_entity).despawn();
                commands.entity(player_entity).despawn();

                game_over_event_writer.send(GameOver { score: score.value });

                break;
            }
        }
    }
    // commands.spawn((
    //     AudioBundle {
    //         source: asset_server.load("audio/sfx_lose.ogg"),
    //         settings: PlaybackSettings::ONCE,
    //     },
    //     BounceSound,
    // ));
}

fn enemy_spawn_cycle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
) {
    enemy_spawn_timer.timer.tick(time.delta());

    if enemy_spawn_timer.timer.finished() {
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                texture: asset_server.load("png/Enemies/enemyRed3.png"),
                ..Default::default()
            },
            Enemy {
                direction: Vec2::new(1.0, 1.0).normalize(),
                row: 0,
                col: 0,
            },
        ));
    }
}

fn bullet_spawn(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.get_single() {
            let translation = player_transform.translation;

            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(translation.x, translation.y, 0.0),
                    texture: asset_server.load("png/Lasers/laserGreen05.png"),
                    ..Default::default()
                },
                Bullet {},
            ));

            commands.spawn((
                AudioBundle {
                    source: asset_server.load("audio/sfx_laser1.ogg"),
                    settings: PlaybackSettings::ONCE,
                },
                BulletSpawnSound,
            ));
        }
    }
}

fn bullet_movement(mut bullet_query: Query<&mut Transform, With<Bullet>>, time: Res<Time>) {
    for mut bullet_transform in bullet_query.iter_mut() {
        let direction = Vec3::new(0.0, 1.0, 0.0);
        bullet_transform.translation += direction * BULLET_SPEED * time.delta_seconds();
    }
}

fn bullet_bounds(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
) {
    let window = window_query.get_single().unwrap();
    let half_bullet_size = BULLET_SIZE / 2.0;

    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        let translation = bullet_transform.translation;

        if translation.y > window.height() - half_bullet_size {
            println!("BULLET OOB");
            commands.entity(bullet_entity).despawn();
        }
    }
}

fn bullet_hit_enemy(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let distance = bullet_transform
                .translation
                .distance(enemy_transform.translation);
            let bullet_radius = BULLET_SIZE / 2.0;
            let enemy_radius = ENEMY_SIZE / 2.0;

            if distance < bullet_radius + enemy_radius {
                println!("BULLET HIT");

                score.value += 1;

                commands.spawn((
                    AudioBundle {
                        source: asset_server.load("audio/sfx_laser2.ogg"),
                        settings: PlaybackSettings::ONCE,
                    },
                    BulletHitSound,
                ));

                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
            }
        }
    }
    // commands.spawn((
    //     AudioBundle {
    //         source: asset_server.load("audio/sfx_lose.ogg"),
    //         settings: PlaybackSettings::ONCE,
    //     },
    //     BounceSound,
    // ));
}

fn update_score(score: Res<Score>) {
    if score.is_changed() {
        println!("{}", score.value);
    }
}

fn handle_game_over(mut commands: Commands, mut game_over_event_reader: EventReader<GameOver>) {
    for event in game_over_event_reader.iter() {
        println!("GAME OVER. Final Score: {}", event.score);
        dbg!("SimulationState::MainMenu");
        commands.insert_resource(NextState(Some(AppState::MainMenu)));
    }
}

fn toggle_simulation(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    simulation_state: Res<State<SimulationState>>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        if simulation_state.get() == &SimulationState::Running {
            dbg!("SimulationState::Paused");
            commands.insert_resource(NextState(Some(SimulationState::Paused)));
        }
        if simulation_state.get() == &SimulationState::Paused {
            dbg!("SimulationState::Running");
            commands.insert_resource(NextState(Some(SimulationState::Running)));
        }
    }
}
