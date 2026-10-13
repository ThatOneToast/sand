//! Player-scoped subscriptions backed by the shared observation planner.

use super::{EventDispatchRepresentation, SandEventDispatch};
use crate::TransitionKind;
use crate::condition::Condition;
use crate::entity::{EntityContext, PlayerKind, StatCurve};
use crate::transition::observation::{Observation, ObservationSource};
use std::collections::BTreeMap;

/// Owned transport for generated event registration. Not an authoring endpoint.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub struct OwnedObservationDispatch {
    pub(crate) tracker_id: String,
    pub(crate) description: String,
    pub(crate) observation: Observation,
    pub(crate) kind: TransitionKind,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::events::ObservationEventDispatch",
    module = "sand::events",
    summary = "Selects a transition for a canonical player-scoped condition or numeric value.",
    context = "Construct with SandEventDispatch::observe_players. Condition subscriptions select became_true or became_false; StatCurve subscriptions select changed, increased or decreased. Each successful first sample initializes a silent baseline. Identical sources share sampling across handlers in the existing player tick lifecycle.",
    minecraft = "Samples once per player tick and retains compiler-owned per-player score baselines. Missing numeric samples neither dispatch nor replace the last valid baseline. Boolean sources follow Condition semantics. Baselines persist across reload and reconnect.",
    use_when = ["Reacting to a change in typed inventory, living-entity or State values"],
    avoid_when = ["Running every tick while a condition holds; use SandEventDispatch::tick instead"],
    example = "use sand::events::ObservationEventDispatch;",
)]
/// Select a change in a canonical player-scoped value.
///
/// Start with [`SandEventDispatch::observe_players`], then choose a transition.
/// The value is sampled in each player's execution context every tick. The
/// first successful sample initializes silently; baselines persist across
/// reloads and reconnects. Numeric read failure preserves the last valid
/// baseline. A false Condition remains a valid boolean sample.
///
/// Identical sources share one sampler. Source groups run in deterministic
/// order for one player before moving to the next player; handler writes can
/// affect groups sampled later in the same tick. Each source anchors to the
/// player's current position and dimension, including after handler movement.
/// This is not a global snapshot or a source-value payload passed to handlers.
///
/// Only canonical conditions and numeric values expose transition methods:
///
/// ```compile_fail
/// use sand_core::events::SandEventDispatch;
/// let event = SandEventDispatch::observe_players(|_| "not a runtime value").changed();
/// ```
pub struct ObservationEventDispatch<T> {
    source: T,
}

impl SandEventDispatch {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::SandEventDispatch::observe_players",
        aliases = ["sand::prelude::SandEventDispatch::observe_players"],
        module = "sand::events",
        kind = "method",
        summary = "Builds a change subscription from a typed player-context source.",
        context = "The factory runs at export time and returns either a Condition or StatCurve over the supplied player context. Select a boolean or numeric transition on the returned ObservationEventDispatch. Sources are sampled in the existing player tick lifecycle, with a silent initial baseline and shared sampling for identical sources.",
        minecraft = "Observes each player once per tick in their own execution context without adding a separate player scan. Missing numeric samples preserve the prior valid baseline.",
        use_when = ["Implementing a custom SandEvent for changes in typed player inventory or living-entity state"],
        avoid_when = ["Unbounded entity or block polling", "Replacing an available advancement trigger", "Replacing a State flow or dirty system that already handles Sand-owned writes"],
        params(source = "Builds a canonical Condition or StatCurve from the observed player's typed context at export time; it does not emit gameplay actions."),
        returns = "A transition builder for the factory's canonical Condition or StatCurve.",
        example = "use sand::events::SandEventDispatch;\nuse sand::entity::StatCurve;\nfn health_changes() -> SandEventDispatch {\n    SandEventDispatch::observe_players(|player| StatCurve::from(player.living().health())).changed()\n}",
    )]
    /// Observe a canonical value for each player, using the shared tick scan.
    ///
    /// `source` executes at export time. Return a [`Condition`] or [`StatCurve`]
    /// built from the supplied player context, then choose the transition to
    /// dispatch. The factory is for constructing a value, not emitting actions.
    /// Observation storage is per player and the first valid sample is silent.
    pub fn observe_players<T>(
        source: impl FnOnce(EntityContext<PlayerKind>) -> T,
    ) -> ObservationEventDispatch<T> {
        ObservationEventDispatch {
            source: source(EntityContext::new()),
        }
    }
}

fn dispatch(source: ObservationSource, kind: TransitionKind) -> SandEventDispatch {
    let description = format!("player observation {source:?}");
    SandEventDispatch(EventDispatchRepresentation::Tracked(
        OwnedObservationDispatch {
            tracker_id: description.clone(),
            description,
            observation: Observation {
                source,
                declarations: BTreeMap::new(),
            },
            kind,
        },
    ))
}

impl ObservationEventDispatch<Condition> {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ObservationEventDispatch::became_true",
        module = "sand::events",
        kind = "method",
        summary = "Dispatches when the player observation changes from false to true.",
        context = "Completes a player-scoped observation subscription. Compares successful current and previous samples; the first valid sample initializes without dispatch. The returned value is suitable for SandEvent::dispatch.",
        minecraft = "Uses the shared per-player sampler and compiler-owned baseline. Numeric failures preserve the last valid sample; boolean sources use Condition truth semantics.",
        use_when = ["Reacting once to the selected transition"],
        avoid_when = ["Repeatedly running while the source remains unchanged"],
        returns = "The completed custom-event dispatch strategy.",
        example = "use sand::events::ObservationEventDispatch;",
    )]
    /// Dispatch when the observation changes from false to true.
    ///
    /// The first successful sample initializes silently. See
    /// [`SandEventDispatch::observe_players`] for scope and sampling semantics.
    pub fn became_true(self) -> SandEventDispatch {
        dispatch(
            ObservationSource::Boolean(self.source),
            TransitionKind::BecameTrue,
        )
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ObservationEventDispatch::became_false",
        module = "sand::events",
        kind = "method",
        summary = "Dispatches when the player observation changes from true to false.",
        context = "Completes a player-scoped observation subscription. Compares successful current and previous samples; the first valid sample initializes without dispatch. The returned value is suitable for SandEvent::dispatch.",
        minecraft = "Uses the shared per-player sampler and compiler-owned baseline. Numeric failures preserve the last valid sample; boolean sources use Condition truth semantics.",
        use_when = ["Reacting once to the selected transition"],
        avoid_when = ["Repeatedly running while the source remains unchanged"],
        returns = "The completed custom-event dispatch strategy.",
        example = "use sand::events::ObservationEventDispatch;",
    )]
    /// Dispatch when the observation changes from true to false.
    ///
    /// The first successful sample initializes silently. See
    /// [`SandEventDispatch::observe_players`] for scope and sampling semantics.
    pub fn became_false(self) -> SandEventDispatch {
        dispatch(
            ObservationSource::Boolean(self.source),
            TransitionKind::BecameFalse,
        )
    }
}

impl ObservationEventDispatch<StatCurve> {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ObservationEventDispatch::changed",
        module = "sand::events",
        kind = "method",
        summary = "Dispatches when the player observation changes to a different numeric value.",
        context = "Completes a player-scoped observation subscription. Compares successful current and previous samples; the first valid sample initializes without dispatch. The returned value is suitable for SandEvent::dispatch.",
        minecraft = "Uses the shared per-player sampler and compiler-owned baseline. Numeric failures preserve the last valid sample; boolean sources use Condition truth semantics.",
        use_when = ["Reacting once to the selected transition"],
        avoid_when = ["Repeatedly running while the source remains unchanged"],
        returns = "The completed custom-event dispatch strategy.",
        example = "use sand::events::ObservationEventDispatch;",
    )]
    /// Dispatch when the observation changes to a different numeric value.
    ///
    /// The first successful sample initializes silently. See
    /// [`SandEventDispatch::observe_players`] for scope and sampling semantics.
    pub fn changed(self) -> SandEventDispatch {
        dispatch(
            ObservationSource::Numeric(self.source),
            TransitionKind::ScoreChanged,
        )
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ObservationEventDispatch::increased",
        module = "sand::events",
        kind = "method",
        summary = "Dispatches when the player observation changes to a greater numeric value.",
        context = "Completes a player-scoped observation subscription. Compares successful current and previous samples; the first valid sample initializes without dispatch. The returned value is suitable for SandEvent::dispatch.",
        minecraft = "Uses the shared per-player sampler and compiler-owned baseline. Numeric failures preserve the last valid sample; boolean sources use Condition truth semantics.",
        use_when = ["Reacting once to the selected transition"],
        avoid_when = ["Repeatedly running while the source remains unchanged"],
        returns = "The completed custom-event dispatch strategy.",
        example = "use sand::events::ObservationEventDispatch;",
    )]
    /// Dispatch when the observation changes to a greater numeric value.
    ///
    /// The first successful sample initializes silently. See
    /// [`SandEventDispatch::observe_players`] for scope and sampling semantics.
    pub fn increased(self) -> SandEventDispatch {
        dispatch(
            ObservationSource::Numeric(self.source),
            TransitionKind::ScoreIncreased,
        )
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::events::ObservationEventDispatch::decreased",
        module = "sand::events",
        kind = "method",
        summary = "Dispatches when the player observation changes to a smaller numeric value.",
        context = "Completes a player-scoped observation subscription. Compares successful current and previous samples; the first valid sample initializes without dispatch. The returned value is suitable for SandEvent::dispatch.",
        minecraft = "Uses the shared per-player sampler and compiler-owned baseline. Numeric failures preserve the last valid sample; boolean sources use Condition truth semantics.",
        use_when = ["Reacting once to the selected transition"],
        avoid_when = ["Repeatedly running while the source remains unchanged"],
        returns = "The completed custom-event dispatch strategy.",
        example = "use sand::events::ObservationEventDispatch;",
    )]
    /// Dispatch when the observation changes to a smaller numeric value.
    ///
    /// The first successful sample initializes silently. See
    /// [`SandEventDispatch::observe_players`] for scope and sampling semantics.
    pub fn decreased(self) -> SandEventDispatch {
        dispatch(
            ObservationSource::Numeric(self.source),
            TransitionKind::ScoreDecreased,
        )
    }
}
