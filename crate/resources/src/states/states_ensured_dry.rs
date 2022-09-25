use std::marker::PhantomData;

use crate::{
    resources::ts::WithStateDiffs,
    states::{ts::EnsuredDry, States, StatesCurrent},
    Resources,
};

/// Dry-run ensured `State`s for all `ItemSpec`s.
///
/// These are the `State`s collected after `EnsureOpSpec::exec_dry` has been
/// run.
///
/// # Implementors
///
/// You may reference [`StatesEnsuredDry`] after `EnsureCmd::exec_dry` has been
/// run.
///
/// [`Data`]: peace_data::Data
pub type StatesEnsuredDry = States<EnsuredDry>;

/// `Resources` is not used at runtime, but is present to signal this type
/// should only be constructed by `EnsureCmd`.
impl From<(StatesCurrent, &Resources<WithStateDiffs>)> for StatesEnsuredDry {
    fn from((states, _resources): (StatesCurrent, &Resources<WithStateDiffs>)) -> Self {
        Self(states.into_inner(), PhantomData)
    }
}
