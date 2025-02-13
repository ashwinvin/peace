use std::fmt::Debug;

use dyn_clone::DynClone;
use fn_graph::{DataAccess, DataAccessDyn};
use peace_cfg::{async_trait, ItemSpecId, OpCtx};
use peace_resources::{
    resources::ts::{Empty, SetUp},
    states::{StatesCurrent, StatesDesired, StatesSaved},
    type_reg::untagged::BoxDtDisplay,
    Resources,
};

use crate::{
    outcomes::{ItemApplyBoxed, ItemApplyPartialBoxed},
    StatesTypeRegs,
};

/// Internal trait that erases the types from [`ItemSpec`]
///
/// This exists so that different implementations of [`ItemSpec`] can be held
/// under the same boxed trait.
///
/// [`ItemSpec`]: peace_cfg::ItemSpec
#[async_trait(?Send)]
pub trait ItemSpecRt<E>: Debug + DataAccess + DataAccessDyn + DynClone {
    /// Returns the ID of this item spec.
    ///
    /// See [`ItemSpec::id`];
    ///
    /// [`ItemSpec::id`]: peace_cfg::ItemSpec::id
    fn id(&self) -> &ItemSpecId;

    /// Initializes data for the operation's check and `exec` functions.
    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), E>
    where
        E: Debug + std::error::Error;

    /// Registers state types with type registries for deserializing from disk.
    ///
    /// This is necessary to deserialize `StatesSavedFile` and
    /// `StatesDesiredFile`.
    fn state_register(&self, states_type_regs: &mut StatesTypeRegs);

    /// Runs [`ItemSpec::state_clean`].
    ///
    /// [`ItemSpec::state_clean`]: peace_cfg::ItemSpec::state_clean
    async fn state_clean(&self, resources: &Resources<SetUp>) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`ItemSpec::StateCurrentFnSpec`]`::`[`try_exec`].
    ///
    /// [`ItemSpec::StateCurrentFnSpec`]: peace_cfg::ItemSpec::StateCurrentFnSpec
    /// [`try_exec`]: peace_cfg::TryFnSpec::try_exec
    async fn state_current_try_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`ItemSpec::StateCurrentFnSpec`]`::`[`exec`].
    ///
    /// [`ItemSpec::StateCurrentFnSpec`]: peace_cfg::ItemSpec::StateCurrentFnSpec
    /// [`exec`]: peace_cfg::TryFnSpec::exec
    async fn state_current_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`ItemSpec::StateDesiredFnSpec`]`::`[`try_exec`].
    ///
    /// [`ItemSpec::StateDesiredFnSpec`]: peace_cfg::ItemSpec::StateDesiredFnSpec
    /// [`try_exec`]: peace_cfg::TryFnSpec::try_exec
    async fn state_desired_try_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`ItemSpec::StateDesiredFnSpec`]`::`[`exec`].
    ///
    /// [`ItemSpec::StateDesiredFnSpec`]: peace_cfg::ItemSpec::StateDesiredFnSpec
    /// [`exec`]: peace_cfg::TryFnSpec::exec
    async fn state_desired_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Returns the diff between the previous and desired [`State`]s.
    ///
    /// [`State`]: peace_cfg::State
    async fn state_diff_exec_with_states_saved(
        &self,
        resources: &Resources<SetUp>,
        states_saved: &StatesSaved,
        states_desired: &StatesDesired,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Returns the diff between the current and desired [`State`]s.
    ///
    /// [`State`]: peace_cfg::State
    async fn state_diff_exec_with_states_current(
        &self,
        resources: &Resources<SetUp>,
        states_current: &StatesCurrent,
        states_desired: &StatesDesired,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Discovers the information needed for an ensure execution.
    ///
    /// This runs the following functions in order:
    ///
    /// * [`StateCurrentFnSpec::exec`]
    /// * [`StateDesiredFnSpec::exec`]
    /// * [`StateDiffFnSpec::exec`]
    /// * [`ApplyOpSpec::check`]
    ///
    /// [`StateCurrentFnSpec::exec`]: peace_cfg::ItemSpec::StateCurrentFnSpec
    /// [`StateDesiredFnSpec::exec`]: peace_cfg::ItemSpec::StateDesiredFnSpec
    /// [`StateDiffFnSpec::exec`]: peace_cfg::ItemSpec::StateDiffFnSpec
    /// [`ApplyOpSpec::check`]: peace_cfg::ItemSpec::ApplyOpSpec
    async fn ensure_prepare(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)>
    where
        E: Debug + std::error::Error;

    /// Discovers the information needed for a clean execution.
    ///
    /// This runs the following functions in order:
    ///
    /// * [`StateCurrentFnSpec::exec`]
    /// * [`ItemSpec::state_clean`]
    /// * [`StateDiffFnSpec::exec`]
    /// * [`ApplyOpSpec::check`]
    ///
    /// [`StateCurrentFnSpec::exec`]: peace_cfg::ItemSpec::StateCurrentFnSpec
    /// [`ItemSpec::state_clean`]: peace_cfg::ItemSpec::state_clean
    /// [`StateDiffFnSpec::exec`]: peace_cfg::ItemSpec::StateDiffFnSpec
    /// [`ApplyOpSpec::check`]: peace_cfg::ItemSpec::ApplyOpSpec
    async fn clean_prepare(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)>
    where
        E: Debug + std::error::Error;

    /// Dry applies the item from its current state to its desired state.
    ///
    /// This runs the following function in order, passing in the information
    /// collected from [`ensure_prepare`] or [`clean_prepare`]:
    ///
    /// * [`ApplyOpSpec::exec_dry`]
    ///
    /// # Parameters
    ///
    /// * `resources`: The resources in the current execution.
    /// * `item_apply`: The information collected in `self.ensure_prepare`.
    ///
    /// [`ApplyOpSpec::exec_dry`]: peace_cfg::ItemSpec::ApplyOpSpec
    async fn apply_exec_dry(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
        item_apply: &mut ItemApplyBoxed,
    ) -> Result<(), E>
    where
        E: Debug + std::error::Error;

    /// Applies the item from its current state to its desired state.
    ///
    /// This runs the following function in order, passing in the information
    /// collected from [`ensure_prepare`] or [`clean_prepare`]:
    ///
    /// * [`ApplyOpSpec::exec`]
    ///
    /// # Parameters
    ///
    /// * `resources`: The resources in the current execution.
    /// * `item_apply`: The information collected in `self.ensure_prepare`.
    ///
    /// [`ApplyOpSpec::exec`]: peace_cfg::ItemSpec::ApplyOpSpec
    async fn apply_exec(
        &self,
        op_ctx: OpCtx<'_>,
        resources: &Resources<SetUp>,
        item_apply: &mut ItemApplyBoxed,
    ) -> Result<(), E>
    where
        E: Debug + std::error::Error;
}
