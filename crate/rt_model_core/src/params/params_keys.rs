use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Marks the types used for params keys.
///
/// # Design
///
/// This allows types such as `CmdContext` and `ParamsTypeRegs` to have a
/// `ParamsKeys` type parameter without specifying all of the associated type
/// bounds. This means:
///
/// * The code for those types is more understandable.
/// * We reduce the ripple effect of needing each of these associated types
///   propagated to callers who use those types in type / method signatures.
pub trait ParamsKeys {
    type WorkspaceParamsKMaybe: KeyMaybe;
    type ProfileParamsKMaybe: KeyMaybe;
    type FlowParamsKMaybe: KeyMaybe;
}

/// Concrete implementation of `ParamsKeys`.
#[derive(Debug)]
pub struct ParamsKeysImpl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe> {
    /// Marker
    marker: PhantomData<(WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe)>,
}

impl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe>
    ParamsKeysImpl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe>
{
    /// Returns a new `ParamsKeysImpl`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe> Default
    for ParamsKeysImpl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe>
{
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe> ParamsKeys
    for ParamsKeysImpl<WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsKMaybe>
where
    WorkspaceParamsKMaybe: KeyMaybe,
    ProfileParamsKMaybe: KeyMaybe,
    FlowParamsKMaybe: KeyMaybe,
{
    type FlowParamsKMaybe = FlowParamsKMaybe;
    type ProfileParamsKMaybe = ProfileParamsKMaybe;
    type WorkspaceParamsKMaybe = WorkspaceParamsKMaybe;
}

// Supporting types that allow keys to not be explicitly specified
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct KeyUnknown;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct KeyKnown<K>(PhantomData<K>);

pub trait KeyMaybe: Debug {
    type Key: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static;
}

impl KeyMaybe for KeyUnknown {
    type Key = ();
}

impl<K> KeyMaybe for KeyKnown<K>
where
    K: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    type Key = K;
}
