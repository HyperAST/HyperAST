use std::path::Path;

use bevy_ecs::{
    component::Component, entity::Entity, schedule::Schedule, system::Query, world::World,
};

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[test]
fn basic() {
    let mut world = World::new();

    let entity = world
        .spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }))
        .id();

    let entity_ref = world.entity(entity);
    let position = entity_ref.get::<Position>().unwrap();
    let velocity = entity_ref.get::<Velocity>().unwrap();
}

// This system moves each entity with a Position and Velocity component
fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn print_position(query: Query<(Entity, &Position)>) {
    for (entity, position) in &query {
        println!(
            "Entity {:?} is at position: x {}, y {}",
            entity, position.x, position.y
        );
    }
}

#[test]
fn systems() {
    // Create a new empty World to hold our Entities and Components
    let mut world = World::new();

    // Spawn an entity with Position and Velocity components
    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

    // Create a new Schedule, which defines an execution strategy for Systems
    let mut schedule = Schedule::default();

    // Add our system to the schedule
    schedule.add_systems(movement);
    schedule.add_systems(print_position);

    // Run the schedule once. If your app has a "loop", you would run this once per loop
    schedule.run(&mut world);
    schedule.run(&mut world);
    schedule.run(&mut world);
    schedule.run(&mut world);
}

use bevy_ecs::system::RunSystemOnce;
use bevy_ecs::system::{Commands, In};

fn handle_io_errors(
    In(result): In<std::io::Result<()>>,
    // we can also have regular system parameters
    mut commands: Commands,
) {
    if let Err(e) = result {
        eprintln!("I/O error occurred: {}", e);
        // Maybe spawn some error UI or something?
        commands.spawn((/* ... */));
    }
}

fn file_read(w: &mut World) -> std::io::Result<()> {
    Path::new("/tmp/aaaatest.txt").read_dir()?;
    Ok(())
}


mod custom_sys;


use std::num::ParseIntError;

use bevy_ecs::prelude::*;

/// Pipe creates a new system which calls `a`, then calls `b` with the output of `a`
pub fn pipe<A, B, AMarker, BMarker>(
    mut a: A,
    mut b: B,
) -> impl FnMut(In<A::In>, ParamSet<(A::Param, B::Param)>) -> B::Out
where
    // We need A and B to be systems, add those bounds
    A: SystemParamFunction<AMarker>,
    B: SystemParamFunction<BMarker, In = A::Out>,
{
    // The type of `params` is inferred based on the return of this function above
    move |In(a_in), mut params| {
        let shared = a.run(a_in, params.p0());
        b.run(shared, params.p1())
    }
}

#[derive(Resource)]
struct Message(String);

fn parse_message(message: Res<Message>) -> Result<usize, ParseIntError> {
    message.0.parse::<usize>()
}

fn filter(In(result): In<Result<usize, ParseIntError>>) -> Option<usize> {
    result.ok().filter(|&n| n < 100)
}

#[test]
fn simple() {
    let mut world = World::default();
    world.insert_resource(Message("42".to_string()));
    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));

    // pipe the `parse_message_system`'s output into the `filter_system`s input
    let mut piped_system = IntoSystem::into_system(pipe(parse_message, filter));

    piped_system.initialize(&mut world);
    assert_eq!(piped_system.run((), &mut world), Some(42));
    world.run_system_once(piped_system);
    world.run_system_once(print_position);

    // This system moves each entity with a Position and Velocity component
    fn movement2(mut query: Query<(&mut Position, &Velocity)>) -> usize {
        for (mut position, velocity) in &mut query {
            position.x += velocity.x;
            position.y += velocity.y;
        }

        43
    }

    fn print_position2(In(result): In<usize>, query: Query<(Entity, &Position)>) -> Result<(), ()> {
        dbg!(result);
        for (entity, position) in &query {
            println!(
                "Entity {:?} is at position: x {}, y {}",
                entity, position.x, position.y
            );
        }
        Ok(())
    }

    let piped_system = IntoSystem::into_system(print_position2);
    world.run_system_once_with(3, piped_system).unwrap();
    let piped_system = IntoSystem::into_system(pipe(movement2, print_position2));
    world.run_system_once(piped_system).unwrap();
}
