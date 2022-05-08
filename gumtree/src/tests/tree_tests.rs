use specs::{Builder, DispatcherBuilder, World, WorldExt};

use crate::tree::static_analysis::{HelloWorld, Position, UpdatePos, Velocity};

#[test]
fn test_run() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    // Only the second entity will get a position update,
    // because the first one does not have a velocity.
    world
        .create_entity()
        .with(Position { start: 4, end: 7 })
        .build();
    world
        .create_entity()
        .with(Position { start: 2, end: 5 })
        .with(Velocity { x: 5., y: 0. })
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HelloWorld, "hello_world", &[])
        .with(UpdatePos, "update_pos", &["hello_world"])
        .with(HelloWorld, "hello_updated", &["update_pos"])
        .build();

    dispatcher.dispatch(&mut world);
    dispatcher.dispatch(&mut world);
    dispatcher.dispatch(&mut world);
    dispatcher.dispatch(&mut world);
    dispatcher.dispatch(&mut world);
    world.maintain();
}
