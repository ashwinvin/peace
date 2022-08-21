use std::ops::Deref;

use peace_core::ItemSpecId;
use serde::Serialize;
use type_reg::untagged::TypeMap;

use crate::StatesMut;

/// `State`s for all `ItemSpec`s. `TypeMap<ItemSpecId>` newtype.
///
/// # Implementors
///
/// If an `ItemSpec`'s state discovery depends on the `State` of a previous
/// `ItemSpec`, then you should insert the predecessor's state into
/// [`Resources`], and reference that in the subsequent `FnSpec`'s [`Data`]:
///
/// ```rust
/// # use std::path::PathBuf;
/// #
/// # use peace_data::{Data, R};
/// #
/// /// Predecessor `FnSpec::Data`.
/// #[derive(Data, Debug)]
/// pub struct AppUploadParams<'op> {
///     /// Path to the application directory.
///     app_dir: W<'op, PathBuf>,
/// }
///
/// /// Successor `FnSpec::Data`.
/// #[derive(Data, Debug)]
/// pub struct AppInstallParams<'op> {
///     /// Path to the application directory.
///     app_dir: R<'op, PathBuf>,
///     /// Configuration to use.
///     config: W<'op, String>,
/// }
/// ```
///
/// You may reference [`States`] in `EnsureOpSpec::Data` for reading. It is not
/// mutable as `States` must remain unchanged so that all `ItemSpec`s operate
/// over consistent data.
///
/// [`Data`]: peace_data::Data
/// [`Resources`]: crate::Resources
#[derive(Debug, Default, Serialize)]
pub struct States(TypeMap<ItemSpecId>);

impl States {
    /// Returns a new `States` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `States` map with the specified capacity.
    ///
    /// The `States` will be able to hold at least capacity elements
    /// without reallocating. If capacity is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(TypeMap::with_capacity(capacity))
    }

    /// Returns the inner map.
    pub fn into_inner(self) -> TypeMap<ItemSpecId> {
        self.0
    }
}

impl Deref for States {
    type Target = TypeMap<ItemSpecId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TypeMap<ItemSpecId>> for States {
    fn from(type_map: TypeMap<ItemSpecId>) -> Self {
        Self(type_map)
    }
}

impl From<StatesMut> for States {
    fn from(states_mut: StatesMut) -> Self {
        Self(states_mut.into_inner())
    }
}
