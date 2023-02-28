#![allow(clippy::type_complexity)]

use std::fmt::Debug;

use futures::stream::{StreamExt, TryStreamExt};
use peace_resources::{
    internal::{FlowParamsFile, ProfileParamsFile, WorkspaceParamsFile},
    resources::ts::{Empty, SetUp},
    Resources,
};
use peace_rt_model::{
    cmd_context_params::{
        FlowParams, KeyMaybe, ParamsKeys, ParamsTypeRegsBuilder, ProfileParams, WorkspaceParams,
    },
    fn_graph::resman::Resource,
    Error, ItemSpecGraph, StatesTypeRegs, Storage, Workspace, WorkspaceInitializer,
};

pub use self::{
    multi_profile_no_flow_builder::MultiProfileNoFlowBuilder,
    multi_profile_single_flow_builder::MultiProfileSingleFlowBuilder,
    no_profile_no_flow_builder::NoProfileNoFlowBuilder,
    single_profile_no_flow_builder::SingleProfileNoFlowBuilder,
    single_profile_single_flow_builder::SingleProfileSingleFlowBuilder,
};

mod multi_profile_no_flow_builder;
mod multi_profile_single_flow_builder;
mod no_profile_no_flow_builder;
mod single_profile_no_flow_builder;
mod single_profile_single_flow_builder;

/// Collects parameters and initializes values relevant to the built [`CmdCtx`].
#[derive(Debug)]
pub struct CmdCtxBuilder<'ctx, O, ScopeBuilder, PKeys>
where
    PKeys: ParamsKeys + 'static,
{
    /// Workspace that the `peace` tool runs in.
    workspace: &'ctx Workspace,
    /// Output endpoint to return values / errors, and write progress
    /// information to.
    ///
    /// See [`OutputWrite`].
    ///
    /// [`OutputWrite`]: peace_rt_model_core::OutputWrite
    output: &'ctx mut O,
    /// Data held while building `CmdCtx`.
    scope_builder: ScopeBuilder,
    /// Type registries for [`WorkspaceParams`], [`ProfileParams`], and
    /// [`FlowParams`] deserialization.
    ///
    /// [`WorkspaceParams`]: crate::cmd_context_params::WorkspaceParams
    /// [`ProfileParams`]: crate::cmd_context_params::ProfileParams
    /// [`FlowParams`]: crate::cmd_context_params::FlowParams
    params_type_regs_builder: ParamsTypeRegsBuilder<PKeys>,
}

impl<'ctx, O, ScopeBuilder, PKeys> CmdCtxBuilder<'ctx, O, ScopeBuilder, PKeys>
where
    PKeys: ParamsKeys + 'static,
{
    /// Serializes workspace params to storage.
    async fn workspace_params_serialize(
        workspace_params: &WorkspaceParams<<PKeys::WorkspaceParamsKMaybe as KeyMaybe>::Key>,
        storage: &Storage,
        workspace_params_file: &WorkspaceParamsFile,
    ) -> Result<(), Error> {
        WorkspaceInitializer::workspace_params_serialize(
            storage,
            workspace_params,
            workspace_params_file,
        )
        .await?;

        Ok(())
    }

    /// Inserts workspace params into the `Resources` map.
    fn workspace_params_insert(
        mut workspace_params: WorkspaceParams<<PKeys::WorkspaceParamsKMaybe as KeyMaybe>::Key>,
        resources: &mut Resources<Empty>,
    ) {
        workspace_params
            .drain(..)
            .for_each(|(_key, workspace_param)| {
                let workspace_param = workspace_param.into_inner().upcast();
                let type_id = Resource::type_id(&*workspace_param);
                resources.insert_raw(type_id, workspace_param);
            });
    }

    /// Serializes profile params to storage.
    async fn profile_params_serialize(
        profile_params: &ProfileParams<<PKeys::ProfileParamsKMaybe as KeyMaybe>::Key>,
        storage: &Storage,
        profile_params_file: &ProfileParamsFile,
    ) -> Result<(), Error> {
        WorkspaceInitializer::profile_params_serialize(
            storage,
            profile_params,
            profile_params_file,
        )
        .await?;

        Ok(())
    }

    /// Inserts profile params into the `Resources` map.
    fn profile_params_insert(
        mut profile_params: ProfileParams<<PKeys::ProfileParamsKMaybe as KeyMaybe>::Key>,
        resources: &mut Resources<Empty>,
    ) {
        profile_params.drain(..).for_each(|(_key, profile_param)| {
            let profile_param = profile_param.into_inner().upcast();
            let type_id = Resource::type_id(&*profile_param);
            resources.insert_raw(type_id, profile_param);
        });
    }

    /// Serializes flow params to storage.
    async fn flow_params_serialize(
        flow_params: &FlowParams<<PKeys::FlowParamsKMaybe as KeyMaybe>::Key>,
        storage: &Storage,
        flow_params_file: &FlowParamsFile,
    ) -> Result<(), Error> {
        WorkspaceInitializer::flow_params_serialize(storage, flow_params, flow_params_file).await?;

        Ok(())
    }

    /// Inserts flow params into the `Resources` map.
    fn flow_params_insert(
        mut flow_params: FlowParams<<PKeys::FlowParamsKMaybe as KeyMaybe>::Key>,
        resources: &mut Resources<Empty>,
    ) {
        flow_params.drain(..).for_each(|(_key, flow_param)| {
            let flow_param = flow_param.into_inner().upcast();
            let type_id = Resource::type_id(&*flow_param);
            resources.insert_raw(type_id, flow_param);
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn profiles_from_peace_app_dir(
    peace_app_dir: &peace_resources::paths::PeaceAppDir,
    profiles_filter_fn: Option<&dyn Fn(&peace_core::Profile) -> bool>,
) -> Result<Vec<peace_core::Profile>, peace_rt_model::Error> {
    use std::{ffi::OsStr, str::FromStr};

    let mut profiles = Vec::new();
    let mut peace_app_read_dir = tokio::fs::read_dir(peace_app_dir).await.map_err(|error| {
        peace_rt_model::Error::Native(peace_rt_model::NativeError::PeaceAppDirRead {
            peace_app_dir: peace_app_dir.to_path_buf(),
            error,
        })
    })?;
    while let Some(entry) = peace_app_read_dir.next_entry().await.map_err(|error| {
        peace_rt_model::Error::Native(peace_rt_model::NativeError::PeaceAppDirEntryRead {
            peace_app_dir: peace_app_dir.to_path_buf(),
            error,
        })
    })? {
        let file_type = entry.file_type().await.map_err(|error| {
            peace_rt_model::Error::Native(
                peace_rt_model::NativeError::PeaceAppDirEntryFileTypeRead {
                    path: entry.path(),
                    error,
                },
            )
        })?;

        if file_type.is_dir() {
            let entry_path = entry.path();
            if let Some(dir_name) = entry_path.file_name().and_then(OsStr::to_str) {
                // Assume this is a profile directory
                let profile = peace_core::Profile::from_str(dir_name).map_err(|error| {
                    peace_rt_model::Error::Native(
                        peace_rt_model::NativeError::ProfileDirInvalidName {
                            dir_name: dir_name.to_string(),
                            path: entry_path.to_path_buf(),
                            error,
                        },
                    )
                })?;

                if let Some(profiles_filter_fn) = profiles_filter_fn {
                    if !profiles_filter_fn(&profile) {
                        // Exclude any profiles that do not pass the filter
                        continue;
                    }
                }

                profiles.push(profile)
            }

            // Assume non-UTF8 file names are not profile directories
        }
    }

    // Ensure profiles are in a consistent, sensible order.
    profiles.sort();

    Ok(profiles)
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn profiles_from_peace_app_dir(
    _peace_app_dir: &peace_resources::paths::PeaceAppDir,
    _profiles_filter_fn: Option<&dyn Fn(&peace_core::Profile) -> bool>,
) -> Result<Vec<peace_core::Profile>, peace_rt_model::Error> {
    let profiles = Vec::new();

    // Not supported yet -- needs a `Storage` abstraction over both native an web
    // assembly.

    Ok(profiles)
}

/// Registers each item spec's `State` and `StateLogical` for deserialization.
fn states_type_regs<E>(item_spec_graph: &ItemSpecGraph<E>) -> StatesTypeRegs {
    item_spec_graph
        .iter()
        .fold(StatesTypeRegs::new(), |mut states_type_regs, item_spec| {
            item_spec.state_register(&mut states_type_regs);

            states_type_regs
        })
}

async fn item_spec_graph_setup<E>(
    item_spec_graph: &ItemSpecGraph<E>,
    resources: Resources<Empty>,
) -> Result<Resources<SetUp>, E>
where
    E: std::error::Error,
{
    let resources = item_spec_graph
        .stream()
        .map(Ok::<_, E>)
        .try_fold(resources, |mut resources, item_spec| async move {
            item_spec.setup(&mut resources).await?;
            Ok(resources)
        })
        .await?;

    Ok(Resources::<SetUp>::from(resources))
}
