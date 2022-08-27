use std::marker::PhantomData;

use futures::stream::{StreamExt, TryStreamExt};
use peace_resources::{
    resources_type_state::{SetUp, WithStatesDesired},
    Resources, StatesDesired, StatesDesiredMut,
};
use peace_rt_model::{ItemSpecGraph, Workspace};

use crate::BUFFERED_FUTURES_MAX;

#[derive(Debug)]
pub struct StateDesiredCmd<E>(PhantomData<E>);

impl<E> StateDesiredCmd<E>
where
    E: std::error::Error,
{
    /// Runs [`StateDesiredFnSpec`]`::`[`exec`] for each [`ItemSpec`].
    ///
    /// At the end of this function, [`Resources`] will be populated with
    /// [`StatesDesired`].
    ///
    /// If any `StateDesiredFnSpec` needs to read the `State` from a previous
    /// `ItemSpec`, the predecessor should insert a copy / clone of their
    /// desired state into `Resources`, and the successor should references
    /// it in their [`FnSpec::Data`].
    ///
    /// [`exec`]: peace_cfg::StateDesiredFnSpec::exec
    /// [`FnSpec::Data`]: peace_cfg::FnSpec::Data
    /// [`ItemSpec`]: peace_cfg::ItemSpec
    /// [`StatesDesired`]: peace_resources::StatesDesired
    /// [`StateDesiredFnSpec`]: peace_cfg::ItemSpec::StateDesiredFnSpec
    pub async fn exec(
        workspace: Workspace<SetUp, E>,
    ) -> Result<Workspace<WithStatesDesired, E>, E> {
        let (resources, item_spec_graph) = workspace.into_inner();
        let states_desired = Self::exec_internal(&item_spec_graph, &resources).await?;

        let resources = Resources::<WithStatesDesired>::from((resources, states_desired));
        let workspace = Workspace::from((resources, item_spec_graph));
        Ok(workspace)
    }

    /// Runs [`StateDesiredFnSpec`]`::`[`exec`] for each [`ItemSpec`].
    ///
    /// Same as [`Self::exec`], but does not change the type state.
    ///
    /// [`exec`]: peace_cfg::StateDesiredFnSpec::exec
    /// [`ItemSpec`]: peace_cfg::ItemSpec
    /// [`StateDesiredFnSpec`]: peace_cfg::ItemSpec::StateDesiredFnSpec
    pub(crate) async fn exec_internal(
        item_spec_graph: &ItemSpecGraph<E>,
        resources: &Resources<SetUp>,
    ) -> Result<StatesDesired, E> {
        let states_desired_mut = item_spec_graph
            .stream()
            .map(Result::Ok)
            .map_ok(|item_spec| async move {
                let state_desired = item_spec.state_desired_fn_exec(resources).await?;
                Ok((item_spec.id(), state_desired))
            })
            .try_buffer_unordered(BUFFERED_FUTURES_MAX)
            .try_collect::<StatesDesiredMut>()
            .await?;

        Ok(StatesDesired::from(states_desired_mut))
    }
}
