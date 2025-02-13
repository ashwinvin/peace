use std::marker::PhantomData;

#[cfg(feature = "output_progress")]
use peace::cfg::progress::ProgressLimit;
use peace::cfg::{async_trait, ApplyOpSpec, OpCheckStatus, OpCtx, State};

use crate::{
    ShCmd, ShCmdData, ShCmdError, ShCmdExecutionRecord, ShCmdExecutor, ShCmdState, ShCmdStateDiff,
};

/// ApplyOpSpec for the command to execute.
#[derive(Debug)]
pub struct ShCmdApplyOpSpec<Id>(PhantomData<Id>);

#[async_trait(?Send)]
impl<Id> ApplyOpSpec for ShCmdApplyOpSpec<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'op> = ShCmdData<'op, Id>;
    type Error = ShCmdError;
    type State = State<ShCmdState<Id>, ShCmdExecutionRecord>;
    type StateDiff = ShCmdStateDiff;

    async fn check(
        sh_cmd_data: ShCmdData<'_, Id>,
        state_current: &State<ShCmdState<Id>, ShCmdExecutionRecord>,
        state_desired: &State<ShCmdState<Id>, ShCmdExecutionRecord>,
        state_diff: &ShCmdStateDiff,
    ) -> Result<OpCheckStatus, ShCmdError> {
        let state_current_arg = match &state_current.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let state_desired_arg = match &state_desired.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let apply_check_sh_cmd = sh_cmd_data
            .sh_cmd_params()
            .apply_check_sh_cmd()
            .clone()
            .arg(state_current_arg)
            .arg(state_desired_arg)
            .arg(&**state_diff);

        ShCmdExecutor::<Id>::exec(&apply_check_sh_cmd)
            .await
            .and_then(|state| match state.logical {
                ShCmdState::Some { stdout, .. } => match stdout.trim().lines().rev().next() {
                    Some("true") => {
                        #[cfg(not(feature = "output_progress"))]
                        {
                            Ok(OpCheckStatus::ExecRequired)
                        }

                        #[cfg(feature = "output_progress")]
                        Ok(OpCheckStatus::ExecRequired {
                            progress_limit: ProgressLimit::Unknown,
                        })
                    }
                    Some("false") => Ok(OpCheckStatus::ExecNotRequired),
                    _ => Err(ShCmdError::EnsureCheckValueNotBoolean {
                        sh_cmd: apply_check_sh_cmd.clone(),
                        #[cfg(feature = "error_reporting")]
                        sh_cmd_string: format!("{apply_check_sh_cmd}"),
                        stdout: Some(stdout),
                    }),
                },
                _ => Err(ShCmdError::EnsureCheckValueNotBoolean {
                    sh_cmd: apply_check_sh_cmd.clone(),
                    #[cfg(feature = "error_reporting")]
                    sh_cmd_string: format!("{apply_check_sh_cmd}"),
                    stdout: None,
                }),
            })
    }

    async fn exec_dry(
        _op_ctx: OpCtx<'_>,
        sh_cmd_data: ShCmdData<'_, Id>,
        state_current: &State<ShCmdState<Id>, ShCmdExecutionRecord>,
        state_desired: &State<ShCmdState<Id>, ShCmdExecutionRecord>,
        state_diff: &ShCmdStateDiff,
    ) -> Result<Self::State, ShCmdError> {
        // TODO: implement properly
        let state_current_arg = match &state_current.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let state_desired_arg = match &state_desired.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let apply_exec_sh_cmd = sh_cmd_data
            .sh_cmd_params()
            .apply_exec_sh_cmd()
            .clone()
            .arg(state_current_arg)
            .arg(state_desired_arg)
            .arg(&**state_diff);

        ShCmdExecutor::<Id>::exec(&ShCmd::new("echo").arg(format!("{apply_exec_sh_cmd}"))).await
    }

    async fn exec(
        _op_ctx: OpCtx<'_>,
        sh_cmd_data: ShCmdData<'_, Id>,
        state_current: &State<ShCmdState<Id>, ShCmdExecutionRecord>,
        state_desired: &State<ShCmdState<Id>, ShCmdExecutionRecord>,
        state_diff: &ShCmdStateDiff,
    ) -> Result<Self::State, ShCmdError> {
        let state_current_arg = match &state_current.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let state_desired_arg = match &state_desired.logical {
            ShCmdState::None => "",
            ShCmdState::Some { stdout, .. } => stdout.as_ref(),
        };
        let apply_exec_sh_cmd = sh_cmd_data
            .sh_cmd_params()
            .apply_exec_sh_cmd()
            .clone()
            .arg(state_current_arg)
            .arg(state_desired_arg)
            .arg(&**state_diff);

        ShCmdExecutor::<Id>::exec(&apply_exec_sh_cmd).await?;
        ShCmdExecutor::<Id>::exec(sh_cmd_data.sh_cmd_params().state_current_sh_cmd()).await
    }
}
