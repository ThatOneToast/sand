//! The single export-scoped store behind every typed command family's
//! pre-write re-validation registry (#293).
//!
//! # Why this exists
//!
//! Several typed command families ([`crate::blocks`], [`crate::nbt`],
//! [`crate::particles`], [`crate::sound`], [`crate::display`],
//! [`crate::text`], [`crate::effect`], [`crate::inventory`],
//! [`crate::execute_ir`]) render to a `String` but want the *typed* node
//! re-validated against the export's resolved [`CommandProfile`] at the
//! pre-write boundary ([`crate::render::validate_collected_line`]), long
//! after the type has been erased into a function body's line text. Each
//! family therefore retains a `rendered line text -> typed node` side table
//! and looks the line back up at export time.
//!
//! Historically each of those side tables was an independent process-global
//! `OnceLock<Mutex<BTreeMap<String, _>>>`. A `Mutex` makes concurrent access
//! *data-race free*, but it does not give *export isolation*: a single map
//! keyed only by rendered line text is shared by every export in the
//! process, forever. So a line registered by export A stays registered
//! during export B, and if B (or a hand-authored raw command, or an
//! unrelated pack) happens to emit byte-identical text, B's line gets
//! re-validated against A's stale typed node — a false accept or, worse, a
//! false reject of a command B never built with these types at all.
//!
//! # The model
//!
//! State lives in a thread-local *stack of layers*. Registration and lookup
//! always target the **top** layer only:
//!
//! * **Layer 0 (ambient)** — always present. Used when no export is in
//!   progress, e.g. a unit test or a downstream caller that renders a typed
//!   command and immediately calls
//!   [`crate::render::validate_collected_line`] on it. It is per-thread and
//!   never shared, but it is not cleared on any schedule.
//! * **An export layer** — pushed by [`ExportRegistryGuard::enter`] for the
//!   duration of one export and popped by its `Drop`.
//!
//! Because lookup only ever consults the top layer, an in-progress export
//! cannot see the ambient layer's entries, cannot see any other thread's
//! entries, and — since the layer is dropped whole — cannot leak entries
//! into whatever runs next, *including when the export returns `Err` or
//! panics*. Cleanup is by `Drop`, never by a call at the end of the happy
//! path.
//!
//! # Adding a tenth family
//!
//! Declare a zero-sized marker type, implement [`RegistryFamily`] for it,
//! and route all state access through [`with_state`]/[`read_state`] (or the
//! [`register_line`]/[`lookup_line`] helpers for the common
//! `BTreeMap<String, Node>` shape). There is no per-family reset to
//! remember to write and no per-family reset to remember to wire into the
//! export pipeline: a family that keeps its state here is export-scoped by
//! construction, because the state has no other place to live.
//!
//! # Nested and concurrent exports
//!
//! * **Concurrent** exports on different threads are fully isolated —
//!   layers are thread-local, and [`ExportRegistryGuard`] is deliberately
//!   `!Send` so a scope cannot be opened on one thread and closed on
//!   another.
//! * **Nested** (reentrant) exports on one thread are **not supported** and
//!   are diagnosed: a second [`ExportRegistryGuard::enter`] while a scope is
//!   already active returns [`NestedExportError`] instead of silently
//!   sharing, stacking, or clobbering the outer export's registry.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use crate::render::CommandProfile;

/// One typed command family's export-scoped state.
///
/// Implemented by a zero-sized marker type per family. `State` is created
/// with [`Default::default`] the first time the family touches the active
/// layer, and is dropped with that layer.
pub(crate) trait RegistryFamily: 'static {
    /// The family's accumulated state for one export.
    ///
    /// Almost always `BTreeMap<String, SomeTypedNode>` (see
    /// [`register_line`]/[`lookup_line`]); [`crate::execute_ir`] instead
    /// accumulates per-line capability requirements, which is why this is a
    /// free-form associated type rather than a fixed map.
    type State: Default + 'static;
}

/// One layer of the registry stack: at most one `State` per family, keyed by
/// the family marker's [`TypeId`].
///
/// The `HashMap` is only ever accessed by exact key, never iterated, so its
/// (nondeterministic) iteration order cannot leak into export output.
type Layer = HashMap<TypeId, Box<dyn Any>>;

thread_local! {
    /// Index 0 is the ambient layer and is never popped; an active export
    /// pushes exactly one layer on top of it.
    static LAYERS: RefCell<Vec<Layer>> = RefCell::new(vec![Layer::new()]);
}

/// Run `f` against the active layer's state for `F`, creating it if absent.
///
/// `f` must not itself call back into this module — the thread-local is
/// mutably borrowed for the duration. Every caller passes a trivial
/// insert/read closure.
pub(crate) fn with_state<F, R>(f: impl FnOnce(&mut F::State) -> R) -> R
where
    F: RegistryFamily,
{
    LAYERS.with(|layers| {
        let mut layers = layers.borrow_mut();
        let layer = layers
            .last_mut()
            .expect("export registry stack always retains its ambient layer");
        let state = layer
            .entry(TypeId::of::<F>())
            .or_insert_with(|| Box::new(F::State::default()));
        f(state
            .downcast_mut::<F::State>()
            .expect("export registry layer entry has its family's state type"))
    })
}

/// Run `f` against the active layer's state for `F` without creating it.
///
/// `None` means this family registered nothing in the active scope — the
/// caller must treat the line as unrecognized and pass it through, never as
/// invalid.
pub(crate) fn read_state<F, R>(f: impl FnOnce(Option<&F::State>) -> R) -> R
where
    F: RegistryFamily,
{
    LAYERS.with(|layers| {
        let layers = layers.borrow();
        let layer = layers
            .last()
            .expect("export registry stack always retains its ambient layer");
        f(layer.get(&TypeId::of::<F>()).map(|state| {
            state
                .downcast_ref::<F::State>()
                .expect("export registry layer entry has its family's state type")
        }))
    })
}

/// Retain `node` as the typed origin of the rendered line text `line`.
///
/// A later render of the same text in the same scope replaces the entry,
/// matching the previous per-family `insert` behaviour.
pub(crate) fn register_line<F, N>(line: &str, node: N)
where
    F: RegistryFamily<State = BTreeMap<String, N>>,
    N: 'static,
{
    with_state::<F, _>(|state| {
        state.insert(line.to_owned(), node);
    });
}

/// Recover the typed node this scope rendered as `line`, if any.
///
/// `None` is the "unknown line" case — a raw/hand-authored command, a line
/// rendered by an earlier export, or a line rendered on another thread. It
/// must always pass through unvalidated: raw lines stay opaque.
pub(crate) fn lookup_line<F, N>(line: &str) -> Option<N>
where
    F: RegistryFamily<State = BTreeMap<String, N>>,
    N: Clone + 'static,
{
    read_state::<F, _>(|state| state.and_then(|state| state.get(line).cloned()))
}

/// Re-validate the typed node registered for `line` against `profile`.
///
/// The shared body of all eight map-shaped families'
/// `validate_registered_line`.
pub(crate) fn validate_registered_line<F, N>(
    line: &str,
    profile: &CommandProfile,
    validate: impl FnOnce(&N, &CommandProfile) -> crate::error::CommandResult<()>,
) -> crate::error::CommandResult<()>
where
    F: RegistryFamily<State = BTreeMap<String, N>>,
    N: Clone + 'static,
{
    match lookup_line::<F, N>(line) {
        Some(node) => validate(&node, profile),
        None => Ok(()),
    }
}

/// Returned by [`ExportRegistryGuard::enter`] when a scope is already active
/// on this thread.
///
/// Nested exports are not supported: the inner export would either observe
/// the outer export's rendered lines (cross-contamination, exactly the bug
/// this module exists to prevent) or discard them on its way out. Sand's
/// exporter is a single non-reentrant pass, so this is a caller bug and is
/// reported as one rather than papered over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct NestedExportError;

impl NestedExportError {
    /// The stable diagnostic code for this error.
    pub const CODE: &'static str = "SAND-EXPORT-REGISTRY-NESTED";
}

impl std::fmt::Display for NestedExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] a typed command export scope is already active on this thread; \
             nested/reentrant exports are not supported because the inner export \
             would observe or discard the outer export's registered command lines",
            Self::CODE
        )
    }
}

impl std::error::Error for NestedExportError {}

/// RAII scope for one export's typed command registries.
///
/// Construct once at the very start of an export and hold it for the whole
/// export body. Every typed command line rendered on this thread while it
/// lives is registered into this scope, and every entry is discarded when
/// it drops — on the happy path, on an early `Err` return, and on unwind.
///
/// Deliberately `!Send`: a scope belongs to the thread that opened it.
///
/// ```
/// use sand_commands::export_registry::ExportRegistryGuard;
///
/// let scope = ExportRegistryGuard::enter().expect("no export in progress");
/// // ... render and validate typed command lines ...
/// assert!(ExportRegistryGuard::enter().is_err(), "nesting is diagnosed");
/// drop(scope);
/// // The scope's registrations are gone; a fresh export starts clean.
/// assert!(ExportRegistryGuard::enter().is_ok());
/// ```
#[derive(Debug)]
pub struct ExportRegistryGuard {
    /// Makes the guard `!Send` (and `!Sync`) without an unstable opt-out.
    _not_send: PhantomData<*const ()>,
}

impl ExportRegistryGuard {
    /// Begin an export scope on this thread.
    ///
    /// # Errors
    ///
    /// Returns [`NestedExportError`] if a scope is already active on this
    /// thread. Exports on other threads are unaffected.
    pub fn enter() -> Result<Self, NestedExportError> {
        LAYERS.with(|layers| {
            let mut layers = layers.borrow_mut();
            if layers.len() > 1 {
                return Err(NestedExportError);
            }
            layers.push(Layer::new());
            Ok(Self {
                _not_send: PhantomData,
            })
        })
    }

    /// Whether an export scope is currently active on this thread.
    #[must_use]
    pub fn is_active() -> bool {
        LAYERS.with(|layers| layers.borrow().len() > 1)
    }
}

impl Drop for ExportRegistryGuard {
    fn drop(&mut self) {
        LAYERS.with(|layers| {
            // `try_borrow_mut` rather than `borrow_mut`: `Drop` runs during
            // unwind, and panicking while already panicking aborts the
            // process. A guard can only be dropped from outside a
            // `with_state`/`read_state` closure (all of which are
            // non-panicking, panic-free bodies), so this branch is
            // unreachable in practice — it is here so an unforeseen path
            // degrades to a leaked layer rather than an abort.
            if let Ok(mut layers) = layers.try_borrow_mut()
                && layers.len() > 1
            {
                layers.pop();
            }
        });
    }
}

#[cfg(test)]
#[path = "export_registry_families.rs"]
mod family_coverage;

#[cfg(test)]
mod tests {
    use super::*;

    struct AlphaFamily;
    impl RegistryFamily for AlphaFamily {
        type State = BTreeMap<String, u32>;
    }

    struct BetaFamily;
    impl RegistryFamily for BetaFamily {
        type State = BTreeMap<String, u32>;
    }

    #[test]
    fn an_export_scope_hides_ambient_entries_and_discards_its_own() {
        register_line::<AlphaFamily, _>("shared line", 1);
        assert_eq!(lookup_line::<AlphaFamily, u32>("shared line"), Some(1));

        {
            let _scope = ExportRegistryGuard::enter().unwrap();
            assert_eq!(
                lookup_line::<AlphaFamily, u32>("shared line"),
                None,
                "an export must not observe entries registered outside it"
            );
            register_line::<AlphaFamily, _>("shared line", 2);
            assert_eq!(lookup_line::<AlphaFamily, u32>("shared line"), Some(2));
        }

        assert_eq!(
            lookup_line::<AlphaFamily, u32>("shared line"),
            Some(1),
            "the ambient layer survives an export scope unchanged"
        );
    }

    #[test]
    fn sequential_scopes_never_share_entries() {
        {
            let _scope = ExportRegistryGuard::enter().unwrap();
            register_line::<AlphaFamily, _>("line", 1);
        }
        let _scope = ExportRegistryGuard::enter().unwrap();
        assert_eq!(lookup_line::<AlphaFamily, u32>("line"), None);
    }

    #[test]
    fn families_do_not_share_a_key_space() {
        let _scope = ExportRegistryGuard::enter().unwrap();
        register_line::<AlphaFamily, _>("line", 1);
        assert_eq!(lookup_line::<BetaFamily, u32>("line"), None);
    }

    #[test]
    fn a_panicking_scope_still_clears() {
        let result = std::panic::catch_unwind(|| {
            let _scope = ExportRegistryGuard::enter().unwrap();
            register_line::<AlphaFamily, _>("panic line", 7);
            panic!("export blew up");
        });
        assert!(result.is_err());
        assert!(!ExportRegistryGuard::is_active());

        let _scope = ExportRegistryGuard::enter().unwrap();
        assert_eq!(lookup_line::<AlphaFamily, u32>("panic line"), None);
    }

    #[test]
    fn nesting_is_diagnosed_not_silently_shared() {
        let _scope = ExportRegistryGuard::enter().unwrap();
        assert_eq!(ExportRegistryGuard::enter().unwrap_err(), NestedExportError);
        assert!(
            ExportRegistryGuard::enter()
                .unwrap_err()
                .to_string()
                .contains(NestedExportError::CODE)
        );
    }

    #[test]
    fn scopes_on_separate_threads_are_isolated() {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let (done_tx, done_rx) = std::sync::mpsc::channel::<Option<u32>>();
        let other = std::thread::spawn(move || {
            let _scope = ExportRegistryGuard::enter().unwrap();
            register_line::<AlphaFamily, _>("cross thread", 99);
            // Hold the scope open while the main thread looks the line up.
            rx.recv().unwrap();
            done_tx
                .send(lookup_line::<AlphaFamily, u32>("cross thread"))
                .unwrap();
        });

        let _scope = ExportRegistryGuard::enter().unwrap();
        tx.send(()).unwrap();
        assert_eq!(
            done_rx.recv().unwrap(),
            Some(99),
            "the other thread still sees its own entry"
        );
        assert_eq!(
            lookup_line::<AlphaFamily, u32>("cross thread"),
            None,
            "another thread's concurrently-open scope is invisible here"
        );
        other.join().unwrap();
    }
}
