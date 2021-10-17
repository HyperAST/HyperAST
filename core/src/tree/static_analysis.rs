type SimpleName = String; // max 256 char

// enum Label {
//     SimpleLabel(SimpleName),
//     LongLabel(String),
// }

#[derive(Debug)]
pub enum QualifiedName {
    RawIdentifier {
        name: SimpleName,
    },
    Class {
        name: SimpleName,
        declaring: Box<QualifiedName>,
    },
    Interface {
        name: SimpleName,
        declaring: Box<QualifiedName>,
    },
}

pub type Reference = QualifiedName;

#[derive(Debug)]
pub struct Declaration {
    pub(crate) name: QualifiedName,
    pub(crate) typ: QualifiedName,
}

#[derive(Debug)]
pub(crate) struct Position {
    pub(crate) start: u32,
    pub(crate) end: u32,
}

#[derive(Debug)]
pub(crate) struct Velocity {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

use specs::{
    storage::BTreeStorage, Builder, Component, DispatcherBuilder, ReadStorage, System, VecStorage,
    World, WorldExt, WriteStorage,
};

use super::tree::Type;

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

impl Component for Declaration {
    type Storage = BTreeStorage<Self>;
}

impl Component for Reference {
    type Storage = BTreeStorage<Self>;
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

impl Component for Type {
    type Storage = VecStorage<Self>;
}
pub(crate) struct HelloWorld;

impl<'a> System<'a> for HelloWorld {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, position: Self::SystemData) {
        use specs::Join;

        for position in position.join() {
            println!("Hello, {:?}", &position);
        }
    }
}

pub(crate) struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        use specs::Join;
        // for (vel, pos) in (&vel, &mut pos).join() {
        //     pos.start += vel.x * 0.05;
        //     pos.end += vel.y * 0.05;
        // }
    }
}
