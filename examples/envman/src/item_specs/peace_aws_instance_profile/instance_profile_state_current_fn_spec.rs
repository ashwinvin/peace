use std::marker::PhantomData;

use aws_sdk_iam::{error::GetInstanceProfileErrorKind, types::SdkError};
use peace::cfg::{async_trait, state::Generated, OpCtx, TryFnSpec};

use crate::item_specs::peace_aws_instance_profile::{
    model::InstanceProfileIdAndArn, InstanceProfileData, InstanceProfileError, InstanceProfileState,
};

#[cfg(feature = "output_progress")]
use peace::cfg::progress::ProgressMsgUpdate;

/// Reads the current state of the instance profile state.
#[derive(Debug)]
pub struct InstanceProfileStateCurrentFnSpec<Id>(PhantomData<Id>);

#[async_trait(?Send)]
impl<Id> TryFnSpec for InstanceProfileStateCurrentFnSpec<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'op> = InstanceProfileData<'op, Id>;
    type Error = InstanceProfileError;
    type Output = InstanceProfileState;

    async fn try_exec(
        op_ctx: OpCtx<'_>,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Option<Self::Output>, InstanceProfileError> {
        Self::exec(op_ctx, data).await.map(Some)
    }

    async fn exec(
        op_ctx: OpCtx<'_>,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Self::Output, InstanceProfileError> {
        let client = data.client();
        let name = data.params().name();
        let path = data.params().path();

        #[cfg(not(feature = "output_progress"))]
        let _op_ctx = op_ctx;
        #[cfg(feature = "output_progress")]
        let progress_sender = &op_ctx.progress_sender;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from(
            "fetching instance profile",
        )));
        let get_instance_profile_result = client
            .get_instance_profile()
            .instance_profile_name(name)
            .send()
            .await;
        let instance_profile_opt = match get_instance_profile_result {
            Ok(get_instance_profile_output) => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "instance profile fetched",
                )));

                let instance_profile = get_instance_profile_output.instance_profile().expect(
                    "Expected instance profile to be some when get_instance_profile is successful.",
                );

                let instance_profile_name = instance_profile
                    .instance_profile_name()
                    .expect(
                        "Expected instance profile name to be Some when \
                        get_instance_profile is successful.",
                    )
                    .to_string();
                let instance_profile_path = instance_profile
                    .path()
                    .expect(
                        "Expected instance profile path to be Some when \
                        get_instance_profile is successful.",
                    )
                    .to_string();
                let instance_profile_id = instance_profile
                    .instance_profile_id()
                    .expect(
                        "Expected instance profile id to be Some when \
                        get_instance_profile is successful.",
                    )
                    .to_string();
                let instance_profile_arn = instance_profile
                    .arn()
                    .expect(
                        "Expected instance profile ARN to be Some when \
                        get_instance_profile is successful.",
                    )
                    .to_string();
                let instance_profile_id_and_arn =
                    InstanceProfileIdAndArn::new(instance_profile_id, instance_profile_arn);

                let role_associated = instance_profile
                    .roles()
                    .and_then(|roles| roles.first())
                    .is_some();

                Some((
                    instance_profile_name,
                    instance_profile_path,
                    instance_profile_id_and_arn,
                    role_associated,
                ))
            }
            Err(error) => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "instance profile not fetched",
                )));

                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::item_specs::aws_error_desc!(&error);

                match &error {
                    SdkError::ServiceError(service_error) => match service_error.err().kind {
                        GetInstanceProfileErrorKind::NoSuchEntityException(_) => None,
                        _ => {
                            return Err(InstanceProfileError::InstanceProfileGetError {
                                instance_profile_name: name.to_string(),
                                instance_profile_path: path.to_string(),
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            });
                        }
                    },
                    _ => {
                        return Err(InstanceProfileError::InstanceProfileGetError {
                            instance_profile_name: name.to_string(),
                            instance_profile_path: path.to_string(),
                            #[cfg(feature = "error_reporting")]
                            aws_desc,
                            #[cfg(feature = "error_reporting")]
                            aws_desc_span,
                            error,
                        });
                    }
                }
            }
        };

        if let Some((
            instance_profile_name,
            instance_profile_path,
            instance_profile_id_and_arn,
            role_associated,
        )) = instance_profile_opt
        {
            let state_current = InstanceProfileState::Some {
                name: instance_profile_name,
                path: instance_profile_path,
                instance_profile_id_and_arn: Generated::Value(instance_profile_id_and_arn),
                role_associated,
            };

            Ok(state_current)
        } else {
            Ok(InstanceProfileState::None)
        }
    }
}
