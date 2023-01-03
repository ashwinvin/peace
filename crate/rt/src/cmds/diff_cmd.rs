use std::marker::PhantomData;

use futures::{StreamExt, TryStreamExt};
use peace_resources::{
    internal::StateDiffsMut,
    resources::ts::{SetUp, WithStatesSavedAndDesired, WithStatesSavedDiffs},
    states::StateDiffs,
    Resources,
};
use peace_rt_model::{CmdContext, Error, ItemSpecGraph, OutputWrite, StatesTypeRegs};

use crate::cmds::sub::{StatesDesiredReadCmd, StatesSavedReadCmd};

#[derive(Debug)]
pub struct DiffCmd<E, O, PO>(PhantomData<(E, O, PO)>);

impl<E, O, PO> DiffCmd<E, O, PO>
where
    E: std::error::Error + From<Error> + Send,
    O: OutputWrite<E>,
{
    /// Runs [`StateDiffFnSpec`]` for each [`ItemSpec`].
    ///
    /// At the end of this function, [`Resources`] will be populated with
    /// [`StatesCurrent`], [`StatesDesired`], and [`StateDiffs`].
    ///
    /// [`ItemSpec`]: peace_cfg::ItemSpec
    /// [`StatesCurrent`]: peace_resources::StatesCurrent
    /// [`StatesDesired`]: peace_resources::StatesDesired
    /// [`StatesRw`]: peace_resources::StatesRw
    /// [`StateDiffFnSpec`]: peace_cfg::ItemSpec::StateDiffFnSpec
    /// [`StateDesiredFnSpec`]: peace_cfg::ItemSpec::StateDesiredFnSpec
    pub async fn exec(
        cmd_context: CmdContext<'_, E, O, PO, SetUp>,
    ) -> Result<CmdContext<'_, E, O, PO, WithStatesSavedDiffs>, E> {
        let CmdContext {
            workspace,
            item_spec_graph,
            output,
            progress_output,
            resources,
            states_type_regs,
            ..
        } = cmd_context;

        let state_diffs_result =
            Self::exec_internal_with_states_saved(item_spec_graph, resources, &states_type_regs)
                .await;

        match state_diffs_result {
            Ok(resources) => {
                {
                    let state_diffs = resources.borrow::<StateDiffs>();
                    output.write_state_diffs(&state_diffs).await?;
                }

                let cmd_context = CmdContext::from((
                    workspace,
                    item_spec_graph,
                    output,
                    progress_output,
                    resources,
                    states_type_regs,
                ));
                Ok(cmd_context)
            }
            Err(e) => {
                output.write_err(&e).await?;
                Err(e)
            }
        }
    }

    /// Returns `StateDiffs` between the saved and desired states on disk.
    ///
    /// This also updates `Resources` from `SetUp` to
    /// `WithStatesCurrentAndDesired`.
    pub(crate) async fn exec_internal_with_states_saved(
        item_spec_graph: &ItemSpecGraph<E>,
        mut resources: Resources<SetUp>,
        states_type_regs: &StatesTypeRegs,
    ) -> Result<Resources<WithStatesSavedDiffs>, E> {
        let states_saved = StatesSavedReadCmd::<E, O, PO>::exec_internal(
            &mut resources,
            states_type_regs.states_current_type_reg(),
        )
        .await?;
        let states_desired = StatesDesiredReadCmd::<E, O, PO>::exec_internal(
            &mut resources,
            states_type_regs.states_desired_type_reg(),
        )
        .await?;

        let resources =
            Resources::<WithStatesSavedAndDesired>::from((resources, states_saved, states_desired));
        let resources_ref = &resources;
        let state_diffs = {
            let state_diffs_mut = item_spec_graph
                .stream()
                .map(Result::<_, E>::Ok)
                .and_then(|item_spec| async move {
                    Ok((
                        item_spec.id(),
                        item_spec
                            .state_diff_exec_with_states_saved(resources_ref)
                            .await?,
                    ))
                })
                .try_collect::<StateDiffsMut>()
                .await?;

            StateDiffs::from(state_diffs_mut)
        };

        let resources = Resources::<WithStatesSavedDiffs>::from((resources, state_diffs));
        Ok(resources)
    }
}
