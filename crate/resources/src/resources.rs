use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    resources::ts::{
        Cleaned, CleanedDry, Empty, Ensured, EnsuredDry, SetUp, WithStatesCurrent,
        WithStatesCurrentAndDesired, WithStatesCurrentDiffs, WithStatesDesired, WithStatesSaved,
        WithStatesSavedAndDesired, WithStatesSavedDiffs,
    },
    states::{
        StateDiffs, StatesCleaned, StatesCleanedDry, StatesCurrent, StatesDesired, StatesEnsured,
        StatesEnsuredDry, StatesSaved,
    },
};

pub mod ts;

/// Map of all types at runtime. [`resman::Resources`] newtype.
///
/// This augments the any-map functionality of [`resman::Resources`] with type
/// state, so that it is impossible for developers to pass `Resources` to
/// functions that require particular operations to have executed over the
/// resources beforehand.
///
/// For example, `Resources` must be `setup` before any `TryFnSpec`,
/// `ApplyOpSpec`, or `CleanOpSpec` may execute with it.
///
/// # Type Parameters
///
/// * `TS`: The type state of the `Resources` map.
///
/// [`ItemSpecId`]: peace_cfg::ItemSpecId
#[derive(Debug)]
pub struct Resources<TS> {
    inner: resman::Resources,
    marker: PhantomData<TS>,
}

impl Resources<Empty> {
    /// Returns a new `Resources`.
    pub fn new() -> Self {
        Self {
            inner: resman::Resources::new(),
            marker: PhantomData,
        }
    }
}

impl<TS> Resources<TS> {
    /// Returns the inner [`resman::Resources`].
    pub fn into_inner(self) -> resman::Resources {
        self.inner
    }
}

impl Default for Resources<Empty> {
    fn default() -> Self {
        Self::new()
    }
}

impl<TS> Deref for Resources<TS> {
    type Target = resman::Resources;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<TS> DerefMut for Resources<TS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// For `ItemSpecGraph` after resources have been set up.
impl From<Resources<Empty>> for Resources<SetUp> {
    fn from(resources: Resources<Empty>) -> Self {
        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

// For `StatesSavedReadCmd` after `StatesSaved` have been read.
impl From<(Resources<SetUp>, StatesSaved)> for Resources<WithStatesSaved> {
    fn from((mut resources, states): (Resources<SetUp>, StatesSaved)) -> Self {
        resources.insert(states);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

// For `StatesCurrentDiscoverCmd` after `StatesCurrent` have been discovered.
impl From<(Resources<SetUp>, StatesCurrent)> for Resources<WithStatesCurrent> {
    fn from((mut resources, states): (Resources<SetUp>, StatesCurrent)) -> Self {
        resources.insert(states);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

// For `StatesDesiredDiscoverCmd` after `StatesDesired` have been discovered.
impl From<(Resources<SetUp>, StatesDesired)> for Resources<WithStatesDesired> {
    fn from((mut resources, states_desired): (Resources<SetUp>, StatesDesired)) -> Self {
        resources.insert(states_desired);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<SetUp>, StatesSaved, StatesDesired)> for Resources<WithStatesSavedAndDesired> {
    fn from(
        (mut resources, states_saved, states_desired): (
            Resources<SetUp>,
            StatesSaved,
            StatesDesired,
        ),
    ) -> Self {
        resources.insert(states_saved);
        resources.insert(states_desired);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<SetUp>, StatesCurrent, StatesDesired)>
    for Resources<WithStatesCurrentAndDesired>
{
    fn from(
        (mut resources, states_current, states_desired): (
            Resources<SetUp>,
            StatesCurrent,
            StatesDesired,
        ),
    ) -> Self {
        resources.insert(states_current);
        resources.insert(states_desired);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<WithStatesSavedAndDesired>, StateDiffs)> for Resources<WithStatesSavedDiffs> {
    fn from(
        (mut resources, state_diffs): (Resources<WithStatesSavedAndDesired>, StateDiffs),
    ) -> Self {
        resources.insert(state_diffs);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<WithStatesCurrentAndDesired>, StateDiffs)>
    for Resources<WithStatesCurrentDiffs>
{
    fn from(
        (mut resources, state_diffs): (Resources<WithStatesCurrentAndDesired>, StateDiffs),
    ) -> Self {
        resources.insert(state_diffs);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<SetUp>, StatesEnsuredDry)> for Resources<EnsuredDry> {
    fn from((mut resources, states_ensured_dry): (Resources<SetUp>, StatesEnsuredDry)) -> Self {
        resources.insert(states_ensured_dry);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<SetUp>, StatesEnsured)> for Resources<Ensured> {
    fn from((mut resources, states_ensured): (Resources<SetUp>, StatesEnsured)) -> Self {
        resources.insert(states_ensured);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<WithStatesCurrent>, StatesCleanedDry)> for Resources<CleanedDry> {
    fn from(
        (mut resources, states_cleaned_dry): (Resources<WithStatesCurrent>, StatesCleanedDry),
    ) -> Self {
        resources.insert(states_cleaned_dry);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}

impl From<(Resources<WithStatesCurrent>, StatesCleaned)> for Resources<Cleaned> {
    fn from(
        (mut resources, states_cleaned): (Resources<WithStatesCurrent>, StatesCleaned),
    ) -> Self {
        resources.insert(states_cleaned);

        Self {
            inner: resources.into_inner(),
            marker: PhantomData,
        }
    }
}
