use bevy_rapier2d::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::{thread_rng, Rng};
use bevy::{prelude::*,color::palettes::css::GREY};

#[derive(Component)]
pub struct Asteroids;

pub fn spawn_asteroids(
    mut commands: Commands,
    query: Query<&Asteroids>,
) {
    if query.iter().count() > 0 {
        return;
    }
    let asteroid_count = 8;
    let mut rng = thread_rng();

    for _ in 0..=asteroid_count {
        // Generate a random scale for asteroid size
        let scale = rng.gen_range(0.5..1.5);

        // Create random asteroid shapes using Lyon
        let asteroid_shape = create_random_asteroid_shape(scale);

        // Randomized spawn position and corresponding velocity
        let spawn_choice = rng.gen_range(0..4); // 0 = left, 1 = right, 2 = bottom, 3 = top
        let (position, linvel);

        match spawn_choice {
            0 => {
                // Spawn on the left side (-x), move to the right (+x)
                position = Vec3::new(-500., rng.gen_range(-500. ..=500.), 0.);
                linvel = Vec2::new(rng.gen_range(2.0..5.0), 0.0); // Move right
            }
            1 => {
                // Spawn on the right side (+x), move to the left (-x)
                position = Vec3::new(500., rng.gen_range(-500. ..=500.), 0.);
                linvel = Vec2::new(rng.gen_range(-5.0..-2.0), 0.0); // Move left
            }
            2 => {
                // Spawn at the bottom (-y), move up (+y)
                position = Vec3::new(rng.gen_range(-500. ..=500.), -500., 0.);
                linvel = Vec2::new(0.0, rng.gen_range(2.0..5.0)); // Move up
            }
            _ => {
                // Spawn at the top (+y), move down (-y)
                position = Vec3::new(rng.gen_range(-500. ..=500.), 500., 0.);
                linvel = Vec2::new(0.0, rng.gen_range(-5.0..-2.0)); // Move down
            }
        }

        // Generate random angular velocity
        let angvel = rng.gen_range(-2.0..=2.0);

        // Convert the points from lyon's Polygon to Vec<Vec2> for Rapier
        let points: Vec<Vec2> = asteroid_shape.points.clone();

        // Spawn asteroid entity with RigidBody, Collider, Velocity, and shape
        commands.spawn((

            ShapeBundle {
                path: GeometryBuilder::build_as(&asteroid_shape),
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(position.x, position.y, 0.),
                    ..Default::default()
                },
                ..Default::default()
            },

            
            Stroke::new(Color::from(GREY), 2.0),

            Asteroids,

            RigidBody::Dynamic,
            CollidingEntities::default(),
            Collider::polyline(points.clone(), None), // Approximate size
            ActiveEvents::COLLISION_EVENTS,
            Sensor,
            Velocity {
                linvel,
                angvel,
            },
        ));
    }
}



// Function to create random asteroid shapes
fn create_random_asteroid_shape(scale: f32) -> shapes::Polygon {
    let mut rng = thread_rng();

    // Generate random points to form a polygon (asteroid-like shape)
    let mut points = vec![];
    let point_count = 20.;

    for i in 0..point_count as u8 {
        let angle = i as f32 * std::f32::consts::TAU / point_count;
        let mut point = Vec2::from_angle(angle);

        point *= rng.gen_range(10.0 ..= 50.0);
        point *= scale;

        points.push(point);
    }

    shapes::Polygon {
        points,
        closed: true,
    }
}

const WINDOW_WIDTH: f32 = 1000.;
const WINDOW_HEIGHT: f32 = 1000.;


pub fn move_asteroids(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Velocity), With<Asteroids>>,
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