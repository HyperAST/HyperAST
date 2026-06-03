use crate::store::nodes::{CompoRegister, ErasedHolder, ErasedInserter, Compo};

use super::ByteLen;
use bevy_ecs::archetype::ArchetypeGeneration;
use bevy_ecs::component::{Component, ComponentId};
use bevy_ecs::ptr::Ptr;
use bevy_ecs::storage::SparseSet;
use bevy_ecs::world::{EntityRef, EntityWorldMut, World, WorldId};
use num::ToPrimitive;
use std::any::TypeId;

trait CompressedCompo {
    fn decomp(ptr: impl ErasedHolder, tid: TypeId) -> Self
    where
        Self: Sized;

    fn compressed_insert(self, e: &mut impl ErasedInserter);
    fn components<R: CompoRegister>(backend: &mut R) -> Vec<R::Id>;
}

impl ErasedHolder for Ptr<'_> {
    unsafe fn unerase_ref_unchecked<T: 'static + Compo>(&self, tid: std::any::TypeId) -> Option<&T> {
        if tid == std::any::TypeId::of::<T>() {
            Some(unsafe { self.deref() })
        } else {
            None
        }
    }
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        todo!("{}", std::any::type_name::<T>())
    }
}

impl ErasedInserter for EntityWorldMut<'_> {
    fn insert<T: 'static + Compo>(&mut self, t: T) {
        self.insert(t);
    }
}

impl CompoRegister for World {
    type Id = ComponentId;

    fn register_compo<T: 'static + Compo>(&mut self) -> ComponentId {
        self.init_component::<T>()
    }
}

fn precompute_md_compressed<T: Component + CompressedCompo>(
    e: &mut EntityWorldMut<'_>,
    f: impl Fn(&World, EntityRef) -> T,
) {
    let bundle = f(e.world(), EntityRef::from(&*e));
    bundle.compressed_insert(e);
}

fn get_decompressed<T: Component + CompressedCompo>(
    w: &World,
    assoc: &SparseSet<u32, SparseSet<u32, ComponentId>>,
    e: EntityRef,
) -> Option<T> {
    let component_id = w.components().get_id(TypeId::of::<T>()).unwrap();
    let table_id = e.location().table_id;
    let intern_cid = assoc
        .get(component_id.index().to_u32().unwrap())
        .unwrap()
        .get(table_id.as_u32())
        .unwrap();
    let intern_tid = w
        .components()
        .get_info(*intern_cid)
        .unwrap()
        .type_id()
        .unwrap();
    Some(T::decomp(e.get_by_id(*intern_cid)?, intern_tid))
}
pub struct CompressionRegistry {
    compressed: Vec<Vec<ComponentId>>,
    // (ComponentId, TableId) -> ComponentId
    assoc: SparseSet<u32, SparseSet<u32, ComponentId>>,
    arch_generation: ArchetypeGeneration,
    world_id: WorldId,
}

impl CompressionRegistry {
    pub fn new(world_id: WorldId) -> Self {
        Self {
            compressed: vec![],
            assoc: Default::default(),
            arch_generation: ArchetypeGeneration::initial(),
            world_id,
        }
    }
    pub fn update(&mut self, world: &World) {
        let archs = &world.archetypes()[self.arch_generation..];
        self.arch_generation = world.archetypes().generation();
        for t in archs.iter() {
            for c in &self.compressed {
                let main = c[0];
                for &c in c {
                    if t.contains(c) {
                        dbg!(main, t.table_id());
                        self.assoc
                            .get_or_insert_with(main.index().to_u32().unwrap(), || {
                                Default::default()
                            })
                            .insert(t.table_id().as_u32(), c);
                    }
                }
            }
        }
    }

    pub fn add_components(&mut self, components: Vec<ComponentId>) {
        assert!(self
            .compressed
            .iter()
            .find(|x| components[0] == x[0])
            .is_none());
        self.arch_generation = ArchetypeGeneration::initial();
        self.compressed.push(components);
    }
}

// # trying on byte_len

#[derive(Component)]
#[repr(transparent)]
struct ByteLenU8(u8);
impl ByteLenU8 {
    fn decompresses(&self) -> ByteLen {
        ByteLen(self.0 as usize)
    }
}
#[derive(Component)]
#[repr(transparent)]
struct ByteLenU16(u16);
impl ByteLenU16 {
    fn decompresses(&self) -> ByteLen {
        ByteLen(self.0 as usize)
    }
}
#[derive(Component)]
#[repr(transparent)]
struct ByteLenU32(u32);
impl ByteLenU32 {
    fn decompresses(&self) -> ByteLen {
        ByteLen(self.0 as usize)
    }
}

impl CompressedCompo for ByteLen {
    fn decomp(ptr: impl ErasedHolder, tid: TypeId) -> Self
    where
        Self: Sized,
    {
        unsafe { ptr.unerase_ref_unchecked::<ByteLen>(tid) }
            .cloned()
            .or_else(|| unsafe { ptr.unerase_ref_unchecked::<ByteLenU8>(tid) }.map(ByteLenU8::decompresses))
            .or_else(|| unsafe { ptr.unerase_ref_unchecked::<ByteLenU32>(tid) }.map(ByteLenU32::decompresses))
            .or_else(|| unsafe { ptr.unerase_ref_unchecked::<ByteLenU16>(tid) }.map(ByteLenU16::decompresses))
            .unwrap_or_else(|| unreachable!())
        // if tid == TypeId::of::<ByteLenU8>() {
        //     unsafe { ptr.deref::<ByteLenU8>() }.decompresses()
        // } else if tid == TypeId::of::<ByteLenU16>() {
        //     unsafe { ptr.deref::<ByteLenU16>() }.decompresses()
        // } else if tid == TypeId::of::<ByteLenU32>() {
        //     unsafe { ptr.deref::<ByteLenU32>() }.decompresses()
        // } else if tid == TypeId::of::<ByteLen>() {
        //     unsafe { ptr.deref::<ByteLen>() }.clone()
        // } else {
        //     unreachable!()
        // }
    }

    fn compressed_insert(self, e: &mut impl ErasedInserter) {
        if let Some(x) = self.0.to_u8() {
            e.insert(ByteLenU8(x));
        } else {
            e.insert(self);
        }
    }

    fn components<R: CompoRegister>(register: &mut R) -> Vec<R::Id> {
        vec![
            register.register_compo::<ByteLen>(),
            register.register_compo::<ByteLenU8>(),
            register.register_compo::<ByteLenU16>(),
            register.register_compo::<ByteLenU32>(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::store::nodes::bevy_ecs::{
        md_simple::precompute_byte_len, precompute_md, Children, Lab, Leaf, Node, Type,
    };

    use super::*;

    #[test]
    fn test_compressed_md() {
        // Create a new empty World to hold our Entities and Components
        let mut world = World::new();

        // construction
        let mut l_42 = world.spawn(Leaf {
            ty: Type("number"),
            label: Lab("42"),
        });
        precompute_md(&mut l_42, precompute_byte_len);
        let l_42 = l_42.id();
        let mut op_plus = world.spawn(Type("+"));
        precompute_md(&mut op_plus, precompute_byte_len);
        let op_plus = op_plus.id();
        let mut l_x = world.spawn(Leaf {
            ty: Type("identifier"),
            label: Lab("x"),
        });
        precompute_md(&mut l_x, precompute_byte_len);
        let l_x = l_x.id();
        let mut expr_bin = world.spawn(Node {
            ty: Type("binary_expr"),
            cs: Children(vec![l_42, op_plus, l_x].into()),
        });
        precompute_md_compressed(&mut expr_bin, precompute_byte_len);
        let expr_bin = expr_bin.id();

        let mut assoc = CompressionRegistry::new(world.id());
        assoc.add_components(ByteLen::components(&mut world));
        assoc.update(&world);
        assoc.update(&world);
        dbg!(&assoc.assoc);

        let b_len = get_decompressed::<ByteLen>(&world, &assoc.assoc, world.entity(expr_bin));
        eprintln!("{:?}", b_len);
    }
}
