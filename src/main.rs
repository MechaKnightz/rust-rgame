use asteroid::asteroids::{move_asteroids, spawn_asteroids, Asteroids};
use bevy::{color::palettes::css::{GREY, ORANGE_RED, RED, WHITE}, prelude::*, sprite::{ColorMaterial, MaterialMesh2dBundle}, window::WindowResolution};
use bevy_rapier2d::prelude::*;
use bevy_prototype_lyon::prelude::*;

const WINDOW_WIDTH: f32 = 1000.;
const WINDOW_HEIGHT: f32 = 1000.;

mod asteroid {
    pub mod asteroids;
}

const ROTATION_SPEED: f32 = 5.0;
const MAX_THRUST: f32 = 300.0;

const BULLET_SPEED: f32 = 500.;

pub fn main() {
    let mut app = App::new();
        app.insert_resource(Msaa::Sample4);

        app.insert_resource(AmmoInt{ current_ammo: MAX_AMMO, reload_timer: Timer::from_seconds(RELOAD_TIME, TimerMode::Repeating)});

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                title: "Asteroids".to_string(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }));
        app.add_plugins(ShapePlugin);

        app.add_event::<GameEvents>();

        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    
        app.add_plugins(RapierDebugRenderPlugin::default());

        app.add_systems(Startup, (spawn_camera,spawn_player));
        app.add_systems(Update, (
            player_controls,
            shoot_bullet,move_bullet,
            wrap_player_position,
            ));      
        app.add_systems(Update, (
            spawn_asteroids,
            move_asteroids,

            asteroid_collision_with_player,
            handle_collision_events,
            ));
        
        app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component, Clone, Copy)]
pub struct Player {
    pub thrust: f32,
    pub rotation_speed: f32,
}


#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct ThrustCon(bool);


const MAX_AMMO: u32 = 15;
const RELOAD_TIME: f32 = 1.0; // 1 seconds per bullet

#[derive(Resource)]
struct AmmoInt {
    current_ammo: u32,
    reload_timer: Timer,
}

impl Default for AmmoInt {
    fn default() -> Self {
        AmmoInt {
            current_ammo: MAX_AMMO,
            reload_timer: Timer::from_seconds(RELOAD_TIME, TimerMode::Repeating),
        }
    }
}

fn spawn_player ( 
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let ship_mesh = MaterialMesh2dBundle {
        mesh: bevy::sprite::Mesh2dHandle(meshes.add(Triangle2d::default())),
        material: materials.add(Color::from(ORANGE_RED)),
        transform: Transform::default().with_scale(Vec3::splat(50.)),
        ..Default::default()
    };
   
    let a_triangle = Vec2::new(0.0, 2.0);    
    let b_triangle = Vec2::new(-1.0, -1.0); 
    let c_triangle = Vec2::new(1.0, -1.0);  
    
    commands.spawn(
        ( ship_mesh,
                Player { thrust: 0., rotation_speed: 0.},
                Velocity::zero() , ThrustCon(false),
                RigidBody::Dynamic,
                Collider::triangle(a_triangle, b_triangle, c_triangle),
                Sensor,
                ActiveEvents::COLLISION_EVENTS ));

}

fn player_controls (
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Player, &mut Velocity, &mut Transform, &mut ThrustCon,  &Children)>,
    mut thruster_query: Query<&mut Handle<ColorMaterial>>
) {
    
    for (mut player, mut velocity, mut transform, mut thrust_con, children ) in &mut query {
        // Rotation
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            player.rotation_speed = ROTATION_SPEED;
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            player.rotation_speed = -ROTATION_SPEED;
        } else {
            player.rotation_speed = 0.0;
        }

        // Apply rotation
        transform.rotation *= Quat::from_rotation_z(player.rotation_speed * time.delta_seconds());

        // Thrust (forward and backward)
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            player.thrust = MAX_THRUST;
            thrust_con.0 = true;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            player.thrust = -MAX_THRUST;
            thrust_con.0 = true;
        } else {
            player.thrust = 0.0;
            thrust_con.0 = false;
        }

        // Apply thrust in the direction the ship is facing
        let forward = transform.rotation * Vec3::Y;
        velocity.linvel = forward.truncate() * player.thrust * time.delta_seconds();

        // Movement (translation)
        transform.translation += velocity.linvel.extend(0.0);

        // Change color of the child mesh (thruster) based on movement
        for &child in children.iter() {
            if let Ok(mut thruster_material) = thruster_query.get_mut(child) {
                if thrust_con.0 {
                    *thruster_material = materials.add(Color::from(RED));
                } else {
                    *thruster_material = materials.add(Color::from(GREY));
                }
            }
        }
    }
}

fn shoot_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&Transform, &Velocity), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ammo: ResMut<AmmoInt>, // Add ammo resource
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if ammo.current_ammo > 0 { // Only shoot if ammo > 0
            if let Ok((transform, velocity)) = query.get_single() {
                let bullet_direction = transform.rotation * Vec3::Y;
                let bullet_position = transform.translation + bullet_direction * 50.0;

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::new(5.0)).into(),
                        material: materials.add(Color::from(WHITE)),
                        transform: Transform {
                            translation: bullet_position,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Bullet,
                    Velocity {
                        linvel: (bullet_direction.truncate() * BULLET_SPEED) + velocity.linvel,
                        ..Default::default()
                    },
                ));

                ammo.current_ammo -= 1; // Decrease ammo

                ammo.reload_timer.tick(time.delta());
                if ammo.reload_timer.finished() && ammo.current_ammo < MAX_AMMO {
                    ammo.current_ammo += 1; // Refill one bullet every 2 seconds
                }
            }
        }
    }
}

fn move_bullet(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Velocity), With<Bullet>>,
) {
    for (entity, mut transform, velocity) in &mut query {
        // Move the bullet
        transform.translation += velocity.linvel.extend(0.0) * time.delta_seconds();

        // Despawn bullet if it goes out of bounds (optional)
        if transform.translation.x.abs() > WINDOW_WIDTH / 2.0 || transform.translation.y.abs() > WINDOW_HEIGHT / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn wrap_player_position(mut query: Query<&mut Transform, With<Player>>) {
    for mut transform in query.iter_mut() {

        let half_width = WINDOW_WIDTH / 2.0;
        let half_height = WINDOW_HEIGHT / 2.0;

        // Check x-axis
        if transform.translation.x > half_width {
            transform.translation.x = -half_width;
        } else if transform.translation.x < -half_width {
            transform.translation.x = half_width;
        }

        // Check y-axis
        if transform.translation.y > half_height {
            transform.translation.y = -half_height;
        } else if transform.translation.y < -half_height {
            transform.translation.y = half_height;
        }
    }
}

#[derive(Event, Clone, Copy)]
pub enum GameEvents {
    ShipCollideWithAsteroid(Entity),
}

pub fn asteroid_collision_with_player(
    asteroids: Query<&CollidingEntities, With<Asteroids>>,
    player_query: Query<(Entity, &Player), With<Sensor>>,
    mut events: EventWriter<GameEvents>
) {
    for asteroid in &asteroids {
        for hit in asteroid.iter() {
            if let Ok((player, _)) = player_query.get(hit) {
                events.send(GameEvents::ShipCollideWithAsteroid(player));
            } else {
                // println!("No player entity found for collision");
            }
        }
    }
}


pub fn handle_collision_events(
    mut commands: Commands,
    mut events: EventReader<GameEvents>,
    player_entities: Query<Entity, With<Player>>, 
) {
    for event in events.read() {
        
        match event {
            GameEvents::ShipCollideWithAsteroid(player_entity) => {
                for entity in player_entities.iter() {
                    if entity == *player_entity {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}
