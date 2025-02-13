use std::marker::PhantomData;

#[cfg(feature = "output_progress")]
use peace::cfg::progress::{ProgressLimit, ProgressMsgUpdate, ProgressSender};
use peace::cfg::{async_trait, state::Generated, ApplyOpSpec, OpCheckStatus, OpCtx};

use crate::item_specs::peace_aws_iam_role::{
    model::RoleIdAndArn, IamRoleData, IamRoleError, IamRoleState, IamRoleStateDiff,
};

/// ApplyOpSpec for the instance profile state.
#[derive(Debug)]
pub struct IamRoleApplyOpSpec<Id>(PhantomData<Id>);

impl<Id> IamRoleApplyOpSpec<Id> {
    pub(crate) async fn managed_policy_detach(
        #[cfg(feature = "output_progress")] progress_sender: &ProgressSender<'_>,
        client: &aws_sdk_iam::Client,
        name: &str,
        path: &str,
        managed_policy_arn: &str,
    ) -> Result<(), IamRoleError> {
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("detaching policy")));
        client
            .detach_role_policy()
            .role_name(name)
            .policy_arn(managed_policy_arn)
            .send()
            .await
            .map_err(|error| {
                let role_name = name.to_string();
                let role_path = path.to_string();

                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::item_specs::aws_error_desc!(&error);

                IamRoleError::ManagedPolicyDetachError {
                    role_name,
                    role_path,
                    #[cfg(feature = "error_reporting")]
                    aws_desc,
                    #[cfg(feature = "error_reporting")]
                    aws_desc_span,
                    error,
                }
            })?;
        #[cfg(feature = "output_progress")]
        progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("detaching policy")));
        Ok(())
    }
}

#[async_trait(?Send)]
impl<Id> ApplyOpSpec for IamRoleApplyOpSpec<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'op> = IamRoleData<'op, Id>;
    type Error = IamRoleError;
    type State = IamRoleState;
    type StateDiff = IamRoleStateDiff;

    async fn check(
        _iam_role_data: IamRoleData<'_, Id>,
        state_current: &IamRoleState,
        _state_desired: &IamRoleState,
        diff: &IamRoleStateDiff,
    ) -> Result<OpCheckStatus, IamRoleError> {
        match diff {
            IamRoleStateDiff::Added => {
                let op_check_status = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        OpCheckStatus::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        let progress_limit = ProgressLimit::Steps(2);
                        OpCheckStatus::ExecRequired { progress_limit }
                    }
                };

                Ok(op_check_status)
            }
            IamRoleStateDiff::Removed => {
                let op_check_status = match state_current {
                    IamRoleState::None => OpCheckStatus::ExecNotRequired,
                    IamRoleState::Some {
                        name: _,
                        path: _,
                        role_id_and_arn,
                        managed_policy_attachment,
                    } => {
                        let mut steps_required = 0;
                        if managed_policy_attachment.attached() {
                            steps_required += 1;
                        }
                        if matches!(role_id_and_arn, Generated::Value(_)) {
                            steps_required += 1;
                        }

                        if steps_required == 0 {
                            OpCheckStatus::ExecNotRequired
                        } else {
                            #[cfg(not(feature = "output_progress"))]
                            {
                                OpCheckStatus::ExecRequired
                            }
                            #[cfg(feature = "output_progress")]
                            {
                                let progress_limit = ProgressLimit::Steps(steps_required);
                                OpCheckStatus::ExecRequired { progress_limit }
                            }
                        }
                    }
                };

                Ok(op_check_status)
            }
            IamRoleStateDiff::ManagedPolicyAttachmentModified { .. } => {
                let op_check_status = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        OpCheckStatus::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        // Technically could be 1 or 2, whether we detach an existing before
                        // attaching another, or just attach one.
                        let progress_limit = ProgressLimit::Steps(2);
                        OpCheckStatus::ExecRequired { progress_limit }
                    }
                };

                Ok(op_check_status)
            }
            IamRoleStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => Err(IamRoleError::RoleModificationNotSupported {
                name_diff: name_diff.clone(),
                path_diff: path_diff.clone(),
            }),
            IamRoleStateDiff::InSyncExists | IamRoleStateDiff::InSyncDoesNotExist => {
                Ok(OpCheckStatus::ExecNotRequired)
            }
        }
    }

    async fn exec_dry(
        _op_ctx: OpCtx<'_>,
        _iam_role_data: IamRoleData<'_, Id>,
        _state_current: &IamRoleState,
        state_desired: &IamRoleState,
        _diff: &IamRoleStateDiff,
    ) -> Result<IamRoleState, IamRoleError> {
        Ok(state_desired.clone())
    }

    // Not sure why we can't use this:
    //
    // #[cfg(not(feature = "output_progress"))] _op_ctx: OpCtx<'_>,
    // #[cfg(feature = "output_progress")] op_ctx: OpCtx<'_>,
    //
    // There's an error saying lifetime bounds don't match the trait definition.
    //
    // Likely an issue with the codegen in `async-trait`.
    #[allow(unused_variables)]
    async fn exec(
        op_ctx: OpCtx<'_>,
        data: IamRoleData<'_, Id>,
        state_current: &IamRoleState,
        state_desired: &IamRoleState,
        diff: &IamRoleStateDiff,
    ) -> Result<IamRoleState, IamRoleError> {
        #[cfg(feature = "output_progress")]
        let progress_sender = &op_ctx.progress_sender;

        match diff {
            IamRoleStateDiff::Added => match state_desired {
                IamRoleState::None => {
                    panic!("`IamRoleApplyOpSpec::exec` called with state_desired being None.");
                }
                IamRoleState::Some {
                    name,
                    path,
                    role_id_and_arn: _,
                    managed_policy_attachment,
                } => {
                    let assume_role_policy_document =
                        include_str!("ec2_assume_role_policy_document.json");
                    let client = data.client();

                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from("creating role")));
                    let role_create_output = client
                        .create_role()
                        .role_name(name)
                        .path(path)
                        .assume_role_policy_document(assume_role_policy_document)
                        .send()
                        .await
                        .map_err(|error| {
                            let role_name = name.to_string();

                            #[cfg(feature = "error_reporting")]
                            let (aws_desc, aws_desc_span) =
                                crate::item_specs::aws_error_desc!(&error);

                            IamRoleError::RoleCreateError {
                                role_name,
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            }
                        })?;
                    #[cfg(feature = "output_progress")]
                    progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("role created")));

                    let role = role_create_output
                        .role()
                        .expect("Expected role to be Some when created.");
                    let role_id = role
                        .role_id()
                        .expect("Expected role ID to be Some when created.");
                    let role_arn = role
                        .arn()
                        .expect("Expected role ARN to be Some when created.");

                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from("attaching policy")));
                    let Generated::Value(managed_policy_arn) = managed_policy_attachment.arn() else {
                        unreachable!("Impossible to have an attached managed policy without an ARN.");
                    };
                    client
                        .attach_role_policy()
                        .role_name(name)
                        .policy_arn(managed_policy_arn)
                        .send()
                        .await
                        .map_err(|error| {
                            #[cfg(feature = "error_reporting")]
                            let (aws_desc, aws_desc_span) =
                                crate::item_specs::aws_error_desc!(&error);

                            IamRoleError::ManagedPolicyAttachError {
                                role_name: name.clone(),
                                role_path: path.clone(),
                                managed_policy_arn: managed_policy_attachment.arn().to_string(),
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            }
                        })?;
                    #[cfg(feature = "output_progress")]
                    progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("policy attached")));

                    let state_ensured = IamRoleState::Some {
                        name: name.to_string(),
                        path: path.clone(),
                        role_id_and_arn: Generated::Value(RoleIdAndArn::new(
                            role_id.to_string(),
                            role_arn.to_string(),
                        )),
                        managed_policy_attachment: managed_policy_attachment.clone(),
                    };

                    Ok(state_ensured)
                }
            },
            IamRoleStateDiff::Removed => {
                match state_current {
                    IamRoleState::None => {}
                    IamRoleState::Some {
                        name,
                        path,
                        role_id_and_arn,
                        managed_policy_attachment,
                    } => {
                        let client = data.client();
                        if managed_policy_attachment.attached() {
                            let Generated::Value(managed_policy_arn) = managed_policy_attachment.arn() else {
                                unreachable!("Impossible to have an attached managed policy without an ARN.");
                            };
                            Self::managed_policy_detach(
                                #[cfg(feature = "output_progress")]
                                progress_sender,
                                client,
                                name,
                                path,
                                managed_policy_arn,
                            )
                            .await?;
                        }
                        #[cfg(feature = "output_progress")]
                        progress_sender.tick(ProgressMsgUpdate::Set(String::from("deleting role")));
                        if let Generated::Value(role_id_and_arn) = role_id_and_arn {
                            client
                                .delete_role()
                                .role_name(name)
                                .send()
                                .await
                                .map_err(|error| {
                                    let role_name = name.to_string();
                                    let role_id = role_id_and_arn.id().to_string();
                                    let role_arn = role_id_and_arn.arn().to_string();

                                    #[cfg(feature = "error_reporting")]
                                    let (aws_desc, aws_desc_span) =
                                        crate::item_specs::aws_error_desc!(&error);

                                    IamRoleError::RoleDeleteError {
                                        role_name,
                                        role_id,
                                        role_arn,
                                        #[cfg(feature = "error_reporting")]
                                        aws_desc,
                                        #[cfg(feature = "error_reporting")]
                                        aws_desc_span,
                                        error,
                                    }
                                })?;
                            #[cfg(feature = "output_progress")]
                            progress_sender
                                .inc(1, ProgressMsgUpdate::Set(String::from("role deleted")));
                        }
                    }
                }

                let state_applied = state_desired.clone();
                Ok(state_applied)
            }
            IamRoleStateDiff::InSyncExists | IamRoleStateDiff::InSyncDoesNotExist => {
                unreachable!(
                    "`IamRoleApplyOpSpec::exec` should never be called when state is in sync."
                );
            }
            IamRoleStateDiff::ManagedPolicyAttachmentModified {
                managed_policy_attachment_current,
                managed_policy_attachment_desired,
            } => {
                let IamRoleState::Some {
                        name,
                        path,
                        role_id_and_arn: _,
                        managed_policy_attachment,
                    } = state_desired else {
                        panic!("`IamRoleApplyOpSpec::exec` called with state_desired being None.");
                    };

                let client = data.client();
                if managed_policy_attachment_current.attached() {
                    // Detach it.
                    let Generated::Value(managed_policy_arn) = managed_policy_attachment_current.arn() else {
                        unreachable!("Impossible to have an attached managed policy without an ARN.");
                    };
                    Self::managed_policy_detach(
                        #[cfg(feature = "output_progress")]
                        progress_sender,
                        client,
                        name,
                        path,
                        managed_policy_arn,
                    )
                    .await?;
                }

                if managed_policy_attachment_desired.attached() {
                    let Generated::Value(managed_policy_arn) = managed_policy_attachment_desired.arn() else {
                        unreachable!("Impossible to have an attached managed policy without an ARN.");
                    };
                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from("attaching policy")));
                    client
                        .attach_role_policy()
                        .role_name(name)
                        .policy_arn(managed_policy_arn)
                        .send()
                        .await
                        .map_err(|error| {
                            #[cfg(feature = "error_reporting")]
                            let (aws_desc, aws_desc_span) =
                                crate::item_specs::aws_error_desc!(&error);
                            IamRoleError::ManagedPolicyAttachError {
                                role_name: name.clone(),
                                role_path: path.clone(),
                                managed_policy_arn: managed_policy_attachment.arn().to_string(),
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            }
                        })?;
                    #[cfg(feature = "output_progress")]
                    progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("policy attached")));
                }

                let state_ensured = state_desired.clone();
                Ok(state_ensured)
            }
            IamRoleStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => Err(IamRoleError::NameOrPathModificationNotSupported {
                name_diff: name_diff.clone(),
                path_diff: path_diff.clone(),
            }),
        }
    }
}
