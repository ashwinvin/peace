use futures::FutureExt;
use peace::{
    cmd::scopes::SingleProfileSingleFlowView,
    fmt::presentable::{Heading, HeadingLevel, ListNumbered},
    rt::cmds::{sub::StatesSavedReadCmd, CleanCmd},
    rt_model::output::OutputWrite,
};

use crate::{cmds::EnvCmd, model::AppCycleError};

/// Cleans up (deletes) the environment.
#[derive(Debug)]
pub struct EnvCleanCmd;

impl EnvCleanCmd {
    /// Cleans up (deletes) the environment.
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    /// * `slug`: Username and repository of the application to download.
    /// * `version`: Version of the application to download.
    /// * `url`: URL to override where to download the application from.
    pub async fn run<O>(output: &mut O) -> Result<(), AppCycleError>
    where
        O: OutputWrite<AppCycleError> + Send,
    {
        let states_saved = EnvCmd::run(output, true, |ctx| {
            StatesSavedReadCmd::exec(ctx).boxed_local()
        })
        .await?;
        EnvCmd::run(output, false, |ctx| {
            async move {
                let states_saved_ref = &states_saved;
                let states_cleaned = CleanCmd::exec(ctx, states_saved_ref).await?;

                // TODO: there's a bug with states_cleaned not being up to date after resuming
                // from interruption.
                let states_cleaned_raw_map = &**states_cleaned;

                let SingleProfileSingleFlowView { output, flow, .. } = ctx.view();
                let states_cleaned_presentables = {
                    let states_cleaned_presentables = flow
                        .graph()
                        .iter_insertion()
                        .map(|item_spec| {
                            let item_spec_id = item_spec.id();
                            match states_cleaned_raw_map.get(item_spec_id) {
                                Some(state_cleaned) => (item_spec_id, format!(": {state_cleaned}")),
                                None => (item_spec_id, String::from(": <unknown>")),
                            }
                        })
                        .collect::<Vec<_>>();

                    ListNumbered::new(states_cleaned_presentables)
                };

                output
                    .present(&(
                        Heading::new(HeadingLevel::Level1, "States Cleaned"),
                        states_cleaned_presentables,
                        "\n",
                    ))
                    .await?;

                Ok(())
            }
            .boxed_local()
        })
        .await?;

        Ok(())
    }
}
