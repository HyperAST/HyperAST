use bevy_ecs::{prelude::*, system::RunSystemOnce as _};

#[test]
fn simple() {
    let mut world = World::default();
    world.run_system_once(AAA {
        system_meta: SystemMeta::new::<AAA>(),
    });
}

struct AAA {
    system_meta: SystemMeta,
}

impl System for AAA {
    type In = ();

    type Out = ();

    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.system_meta.name.clone()
    }

    fn component_access(&self) -> &bevy_ecs::query::Access<bevy_ecs::component::ComponentId> {
        &self.system_meta.component_access_set.combined_access()
    }

    fn archetype_component_access(
        &self,
    ) -> &bevy_ecs::query::Access<bevy_ecs::archetype::ArchetypeComponentId> {
        &self.system_meta.archetype_component_access
    }

    fn is_send(&self) -> bool {
        self.system_meta.is_send
    }

    fn is_exclusive(&self) -> bool {
        true
    }

    fn has_deferred(&self) -> bool {
        self.system_meta.has_deferred
    }

    unsafe fn run_unsafe(
        &mut self,
        input: Self::In,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Self::Out {
        unimplemented!("only run in exclusive mode");
        // let change_tick = world.increment_change_tick();

        // // SAFETY:
        // // - The caller has invoked `update_archetype_component_access`, which will panic
        // //   if the world does not match.
        // // - All world accesses used by `F::Param` have been registered, so the caller
        // //   will ensure that there are no data access conflicts.
        // // let params = unsafe {
        // //     F::Param::get_param(
        // //         self.param_state.as_mut().expect(Self::PARAM_MESSAGE),
        // //         &self.system_meta,
        // //         world,
        // //         change_tick,
        // //     )
        // // };
        // // let out = self.func.run(input, params);
        // dbg!(input);
        // self.system_meta.last_run = change_tick;
    }

    fn run(&mut self, input: Self::In, world: &mut World) -> Self::Out {}

    fn apply_deferred(&mut self, world: &mut World) {}

    fn queue_deferred(&mut self, world: bevy_ecs::world::DeferredWorld) {}

    fn initialize(&mut self, _world: &mut World) {}

    fn update_archetype_component_access(
        &mut self,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) {
    }

    fn check_change_tick(&mut self, change_tick: bevy_ecs::component::Tick) {}

    fn get_last_run(&self) -> bevy_ecs::component::Tick {
        self.system_meta.last_run
    }

    fn set_last_run(&mut self, last_run: bevy_ecs::component::Tick) {
        self.system_meta.last_run = last_run
    }
}

/// The metadata of a [`System`].
#[derive(Clone)]
pub struct SystemMeta {
    pub(crate) name: std::borrow::Cow<'static, str>,
    pub(crate) component_access_set:
        bevy_ecs::query::FilteredAccessSet<bevy_ecs::component::ComponentId>,
    pub(crate) archetype_component_access:
        bevy_ecs::query::Access<bevy_ecs::archetype::ArchetypeComponentId>,
    // NOTE: this must be kept private. making a SystemMeta non-send is irreversible to prevent
    // SystemParams from overriding each other
    is_send: bool,
    has_deferred: bool,
    pub(crate) last_run: bevy_ecs::component::Tick,
    #[cfg(feature = "trace")]
    pub(crate) system_span: Span,
    #[cfg(feature = "trace")]
    pub(crate) commands_span: Span,
}

impl SystemMeta {
    pub(crate) fn new<T>() -> Self {
        let name = std::any::type_name::<T>();
        Self {
            name: name.into(),
            archetype_component_access: bevy_ecs::query::Access::default(),
            component_access_set: bevy_ecs::query::FilteredAccessSet::default(),
            is_send: true,
            has_deferred: false,
            last_run: bevy_ecs::component::Tick::new(0),
            #[cfg(feature = "trace")]
            system_span: info_span!("system", name = name),
            #[cfg(feature = "trace")]
            commands_span: info_span!("system_commands", name = name),
        }
    }
}
