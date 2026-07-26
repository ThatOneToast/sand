//! Per-family coverage of the export-scoped registry lifecycle (#293).
//!
//! One generic harness, instantiated once per typed command family, that
//! drives each family's *real* registration path (the same public builder
//! call a pack author writes) and asserts the four lifecycle properties
//! that matter, against that family's own storage:
//!
//! 1. a line rendered inside an export scope is recoverable inside it;
//! 2. it is **not** recoverable in the next scope — the cross-export
//!    staleness this issue is about;
//! 3. it is not recoverable after a scope that *panicked* — cleanup is by
//!    `Drop`, not by reaching the end of the happy path;
//! 4. it is not recoverable from a concurrently-open scope on another
//!    thread.
//!
//! Property 2 is asserted at the registry rather than through a synthesized
//! validation failure so it holds for every family uniformly, including the
//! families whose `validate` is profile-independent and therefore has no
//! "same text, different verdict" construction available. The end-to-end
//! consequence — a stale node changing a later export's verdict — is
//! covered against the real pipeline in
//! `sand-core/tests/export_registry_scope.rs`.

use std::collections::BTreeMap;

use super::{ExportRegistryGuard, RegistryFamily, lookup_line};

/// Whether `line` is registered for `F` in the *active* layer.
///
/// Uniform across families because `execute_ir`'s requirement side table is
/// also keyed by rendered line text — its values are capability lists
/// rather than typed nodes, which this helper is deliberately agnostic to.
fn is_registered<F, N>(line: &str) -> bool
where
    F: RegistryFamily<State = BTreeMap<String, N>>,
    N: Clone + 'static,
{
    lookup_line::<F, N>(line).is_some()
}

/// Instantiate the lifecycle harness for one family.
///
/// `$render` must call the family's ordinary public builder and return the
/// single rendered line it registers.
macro_rules! family_lifecycle_tests {
    ($( $family_mod:ident : $family:ty, $node:ty, $render:expr ; )*) => {
        $(
            mod $family_mod {
                use super::*;

                fn render() -> String {
                    let render_one: fn() -> String = $render;
                    render_one()
                }

                fn registered(line: &str) -> bool {
                    is_registered::<$family, $node>(line)
                }

                #[test]
                fn a_line_rendered_in_a_scope_is_recoverable_in_that_scope() {
                    let _scope = ExportRegistryGuard::enter().unwrap();
                    let line = render();
                    assert!(
                        registered(&line),
                        "family did not register `{line}` into the active export scope"
                    );
                }

                #[test]
                fn a_line_from_a_previous_scope_is_unknown_in_the_next_one() {
                    let line = {
                        let _scope = ExportRegistryGuard::enter().unwrap();
                        let line = render();
                        assert!(registered(&line), "test setup: line was never registered");
                        line
                    };

                    let _next = ExportRegistryGuard::enter().unwrap();
                    assert!(
                        !registered(&line),
                        "stale entry for `{line}` survived into the next export scope"
                    );
                }

                #[test]
                fn a_line_from_a_panicking_scope_is_unknown_afterwards() {
                    let line = render();
                    let result = std::panic::catch_unwind(|| {
                        let _scope = ExportRegistryGuard::enter().unwrap();
                        let line = render();
                        assert!(registered(&line), "test setup: line was never registered");
                        panic!("export failed partway");
                    });
                    assert!(result.is_err(), "test setup: the scope must unwind");
                    assert!(
                        !ExportRegistryGuard::is_active(),
                        "a panicking scope must still be closed"
                    );

                    let _next = ExportRegistryGuard::enter().unwrap();
                    assert!(
                        !registered(&line),
                        "state leaked out of a scope that panicked partway through"
                    );
                }

                #[test]
                fn a_concurrent_scope_on_another_thread_is_invisible() {
                    let (start_tx, start_rx) = std::sync::mpsc::channel::<()>();
                    let (line_tx, line_rx) = std::sync::mpsc::channel::<String>();
                    let (verdict_tx, verdict_rx) = std::sync::mpsc::channel::<bool>();

                    let other = std::thread::spawn(move || {
                        let _scope = ExportRegistryGuard::enter().unwrap();
                        let line = render();
                        line_tx.send(line.clone()).unwrap();
                        // Hold this scope open across the other thread's lookup.
                        start_rx.recv().unwrap();
                        verdict_tx.send(registered(&line)).unwrap();
                    });

                    let line = line_rx.recv().unwrap();
                    let _scope = ExportRegistryGuard::enter().unwrap();
                    assert!(
                        !registered(&line),
                        "a concurrently-open scope on another thread leaked `{line}` into this one"
                    );
                    start_tx.send(()).unwrap();
                    assert!(
                        verdict_rx.recv().unwrap(),
                        "the other thread lost its own registration"
                    );
                    other.join().unwrap();
                }
            }
        )*
    };
}

family_lifecycle_tests! {
    blocks: crate::blocks::BlockLines, crate::blocks::BlockCommandNode, || {
        use crate::render::RenderCommand;
        crate::blocks::SetBlock::new(crate::coord::BlockPos::here(), "minecraft:stone")
            .render(&crate::render::CommandProfile::unprofiled())
            .unwrap()
    };

    nbt: crate::nbt::DataLines, crate::nbt::DataCommand, || {
        crate::nbt::Nbt::storage("regtest:scope")
            .path("counter")
            .set(1_i32)
            .try_render(&crate::render::CommandProfile::unprofiled())
            .unwrap()
    };

    particles: crate::particles::ParticleLines, crate::particles::ParticleCommand, || {
        crate::particles::ParticleBuilder::new(
            crate::particles::Particle::named("minecraft:flame"),
        )
        .points_at(&[[0.0, 64.0, 0.0]])
        .remove(0)
    };

    sound: crate::sound::SoundLines, crate::sound::SoundCommand, || {
        use crate::Build;
        crate::sound::Sound::play("minecraft:entity.experience_orb.pickup")
            .to(crate::selector::Selector::self_())
            .build()
    };

    display: crate::display::DisplayLines, crate::display::DisplayCommand, || {
        crate::display::Title::of(crate::selector::Selector::all_players())
            .title(crate::text::TextComponent::literal("Hello"))
            .build()
            .remove(0)
    };

    text: crate::text::TextLines, crate::text::TextCommand, || {
        use crate::Build;
        crate::text::TextCommand::tellraw(
            crate::selector::Selector::self_(),
            crate::text::TextComponent::literal("scoped"),
        )
        .build()
    };

    effect: crate::effect::EffectLines, crate::effect::EffectCommand, || {
        use crate::Build;
        crate::effect::EffectCommand::give(
            crate::selector::Selector::self_(),
            "minecraft:speed",
        )
        .build()
    };

    inventory: crate::inventory::InventoryLines, crate::inventory::InventoryCommandNode, || {
        crate::inventory::Inventory::of(crate::selector::Selector::self_())
            .set(crate::execute_args::ItemSlot::Hotbar(0), "minecraft:stone")
    };

    execute_ir: crate::execute_ir::ExecuteRequirements, Vec<crate::execute_ir::Requirement>, || {
        crate::execute::Execute::new()
            .if_items(
                crate::selector::Selector::self_(),
                crate::execute_args::ItemSlot::MainHand,
                "minecraft:diamond",
            )
            .run_raw("say found")
    };
}
