use std::marker::PhantomData;

use aws_sdk_s3::types::ByteStream;
use base64::Engine;
#[cfg(feature = "output_progress")]
use peace::cfg::progress::{ProgressLimit, ProgressMsgUpdate};
use peace::cfg::{async_trait, state::Generated, ApplyOpSpec, OpCheckStatus, OpCtx};

use crate::item_specs::peace_aws_s3_object::{
    S3ObjectData, S3ObjectError, S3ObjectState, S3ObjectStateDiff,
};

/// ApplyOpSpec for the S3 object state.
#[derive(Debug)]
pub struct S3ObjectApplyOpSpec<Id>(PhantomData<Id>);

#[async_trait(?Send)]
impl<Id> ApplyOpSpec for S3ObjectApplyOpSpec<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'op> = S3ObjectData<'op, Id>;
    type Error = S3ObjectError;
    type State = S3ObjectState;
    type StateDiff = S3ObjectStateDiff;

    async fn check(
        _s3_object_data: S3ObjectData<'_, Id>,
        state_current: &S3ObjectState,
        _state_desired: &S3ObjectState,
        diff: &S3ObjectStateDiff,
    ) -> Result<OpCheckStatus, S3ObjectError> {
        match diff {
            S3ObjectStateDiff::Added { .. } | S3ObjectStateDiff::ObjectContentModified { .. } => {
                let op_check_status = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        OpCheckStatus::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        let progress_limit = ProgressLimit::Steps(1);
                        OpCheckStatus::ExecRequired { progress_limit }
                    }
                };

                Ok(op_check_status)
            }
            S3ObjectStateDiff::Removed => {
                let op_check_status = match state_current {
                    S3ObjectState::None => OpCheckStatus::ExecNotRequired,
                    S3ObjectState::Some {
                        bucket_name: _,
                        object_key: _,
                        content_md5_hexstr: _,
                        e_tag: _,
                    } => {
                        #[cfg(not(feature = "output_progress"))]
                        {
                            OpCheckStatus::ExecRequired
                        }
                        #[cfg(feature = "output_progress")]
                        {
                            let steps_required = 1;
                            let progress_limit = ProgressLimit::Steps(steps_required);
                            OpCheckStatus::ExecRequired { progress_limit }
                        }
                    }
                };

                Ok(op_check_status)
            }
            S3ObjectStateDiff::BucketNameModified {
                bucket_name_current,
                bucket_name_desired,
            } => Err(S3ObjectError::BucketModificationNotSupported {
                bucket_name_current: bucket_name_current.clone(),
                bucket_name_desired: bucket_name_desired.clone(),
            }),
            S3ObjectStateDiff::ObjectKeyModified {
                object_key_current,
                object_key_desired,
            } => Err(S3ObjectError::S3ObjectModificationNotSupported {
                object_key_current: object_key_current.clone(),
                object_key_desired: object_key_desired.clone(),
            }),
            S3ObjectStateDiff::InSyncExists | S3ObjectStateDiff::InSyncDoesNotExist => {
                Ok(OpCheckStatus::ExecNotRequired)
            }
        }
    }

    async fn exec_dry(
        _op_ctx: OpCtx<'_>,
        _s3_object_data: S3ObjectData<'_, Id>,
        _state_current: &S3ObjectState,
        state_desired: &S3ObjectState,
        _diff: &S3ObjectStateDiff,
    ) -> Result<S3ObjectState, S3ObjectError> {
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
        data: S3ObjectData<'_, Id>,
        state_current: &S3ObjectState,
        state_desired: &S3ObjectState,
        diff: &S3ObjectStateDiff,
    ) -> Result<S3ObjectState, S3ObjectError> {
        #[cfg(feature = "output_progress")]
        let progress_sender = &op_ctx.progress_sender;

        match diff {
            S3ObjectStateDiff::Added | S3ObjectStateDiff::ObjectContentModified { .. } => {
                match state_desired {
                    S3ObjectState::None => {
                        panic!("`S3ObjectApplyOpSpec::exec` called with state_desired being None.");
                    }
                    S3ObjectState::Some {
                        bucket_name,
                        object_key,
                        content_md5_hexstr,
                        e_tag: _,
                    } => {
                        let client = data.client();

                        #[cfg(feature = "output_progress")]
                        progress_sender
                            .tick(ProgressMsgUpdate::Set(String::from("uploading object")));
                        let file_path = data.params().file_path();
                        let Some(content_md5_hexstr) = content_md5_hexstr else {
                            panic!("Content MD5 must be Some as this is calculated from an existent local file.");
                        };
                        let content_md5_b64 = {
                            let bytes = (0..content_md5_hexstr.len())
                                .step_by(2)
                                .map(|index_start| {
                                    &content_md5_hexstr[index_start..index_start + 2]
                                })
                                .map(|byte_hexstr| u8::from_str_radix(byte_hexstr, 16))
                                .try_fold(
                                    Vec::<u8>::with_capacity(content_md5_hexstr.len() / 2),
                                    |mut bytes, byte_result| {
                                        byte_result.map(|byte| {
                                            bytes.push(byte);
                                            bytes
                                        })
                                    },
                                )
                                .map_err(|error| {
                                    let file_path = file_path.to_path_buf();
                                    let bucket_name = bucket_name.clone();
                                    let object_key = object_key.clone();
                                    let content_md5_hexstr = content_md5_hexstr.clone();

                                    S3ObjectError::ObjectContentMd5HexstrParse {
                                        file_path,
                                        bucket_name,
                                        object_key,
                                        content_md5_hexstr,
                                        error,
                                    }
                                })?;
                            base64::engine::general_purpose::STANDARD.encode(bytes)
                        };
                        let put_object_output = client
                            .put_object()
                            .bucket(bucket_name)
                            .key(object_key)
                            .content_md5(content_md5_b64)
                            .metadata("content_md5_hexstr", content_md5_hexstr)
                            .body(ByteStream::from_path(file_path).await.map_err(|error| {
                                let file_path = file_path.to_path_buf();
                                let bucket_name = bucket_name.clone();
                                let object_key = object_key.clone();

                                S3ObjectError::ObjectFileStream {
                                    file_path,
                                    bucket_name,
                                    object_key,
                                    error,
                                }
                            })?)
                            .send()
                            .await
                            .map_err(|error| {
                                let bucket_name = bucket_name.to_string();
                                let object_key = object_key.to_string();

                                #[cfg(feature = "error_reporting")]
                                let (aws_desc, aws_desc_span) =
                                    crate::item_specs::aws_error_desc!(&error);

                                S3ObjectError::S3ObjectUploadError {
                                    bucket_name,
                                    object_key,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc_span,
                                    error,
                                }
                            })?;
                        #[cfg(feature = "output_progress")]
                        progress_sender
                            .inc(1, ProgressMsgUpdate::Set(String::from("object uploaded")));
                        let e_tag = put_object_output
                            .e_tag()
                            .expect("Expected ETag to be some when put_object is successful.")
                            .to_string();

                        let state_applied = S3ObjectState::Some {
                            bucket_name: bucket_name.clone(),
                            object_key: object_key.clone(),
                            content_md5_hexstr: Some(content_md5_hexstr.clone()),
                            e_tag: Generated::Value(e_tag),
                        };

                        Ok(state_applied)
                    }
                }
            }
            S3ObjectStateDiff::Removed => {
                match state_current {
                    S3ObjectState::None => {}
                    S3ObjectState::Some {
                        bucket_name,
                        object_key,
                        content_md5_hexstr: _,
                        e_tag: _,
                    } => {
                        let client = data.client();
                        #[cfg(feature = "output_progress")]
                        progress_sender
                            .tick(ProgressMsgUpdate::Set(String::from("deleting object")));
                        client
                            .delete_object()
                            .bucket(bucket_name)
                            .key(object_key)
                            .send()
                            .await
                            .map_err(|error| {
                                let bucket_name = bucket_name.to_string();
                                let object_key = object_key.to_string();

                                #[cfg(feature = "error_reporting")]
                                let (aws_desc, aws_desc_span) =
                                    crate::item_specs::aws_error_desc!(&error);

                                S3ObjectError::S3ObjectDeleteError {
                                    bucket_name,
                                    object_key,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc_span,
                                    error,
                                }
                            })?;
                        #[cfg(feature = "output_progress")]
                        progress_sender
                            .inc(1, ProgressMsgUpdate::Set(String::from("object deleted")));
                    }
                }

                let state_applied = state_desired.clone();
                Ok(state_applied)
            }
            S3ObjectStateDiff::InSyncExists | S3ObjectStateDiff::InSyncDoesNotExist => {
                unreachable!(
                    "`S3ObjectApplyOpSpec::exec` should never be called when state is in sync."
                );
            }
            S3ObjectStateDiff::BucketNameModified {
                bucket_name_current,
                bucket_name_desired,
            } => Err(S3ObjectError::BucketModificationNotSupported {
                bucket_name_current: bucket_name_current.clone(),
                bucket_name_desired: bucket_name_desired.clone(),
            }),
            S3ObjectStateDiff::ObjectKeyModified {
                object_key_current,
                object_key_desired,
            } => {
                let S3ObjectState::Some {bucket_name, ..} = state_desired else {
                    panic!("`S3ObjectApplyOpSpec::exec` called with state_desired being None.");
                };

                Err(S3ObjectError::ObjectKeyModificationNotSupported {
                    bucket_name: bucket_name.clone(),
                    object_key_current: object_key_current.clone(),
                    object_key_desired: object_key_desired.clone(),
                })
            }
        }
    }
}
