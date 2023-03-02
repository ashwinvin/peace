#![allow(clippy::type_complexity)]

use std::ops::{Deref, DerefMut};

use peace_resources::{
    paths::{PeaceAppDir, PeaceDir, WorkspaceDir},
    Resources,
};
use peace_rt_model::{
    params::{KeyUnknown, ParamsKeys, ParamsKeysImpl},
    Workspace,
};

use crate::{
    ctx::{
        cmd_ctx_builder::{
            MultiProfileNoFlowBuilder, MultiProfileSingleFlowBuilder, NoProfileNoFlowBuilder,
            SingleProfileNoFlowBuilder, SingleProfileSingleFlowBuilder,
        },
        CmdCtxBuilder,
    },
    scopes::{
        type_params::{
            FlowNotSelected, FlowParamsNone, ProfileNotSelected, ProfileParamsNone,
            WorkspaceParamsNone,
        },
        SingleProfileSingleFlow,
    },
};

/// Information needed to execute a command.
///
/// Importantly, as commands have different purposes, different command scopes
/// exist to cater for each kind of command. This means the data available in a
/// command context differs per scope, to accurately reflect what is available.
#[derive(Debug)]
pub struct CmdCtx<'ctx, O, Scope> {
    /// Output endpoint to return values / errors, and write progress
    /// information to.
    ///
    /// See [`OutputWrite`].
    ///
    /// [`OutputWrite`]: peace_rt_model_core::OutputWrite
    pub(crate) output: &'ctx mut O,
    /// Workspace that the `peace` tool runs in.
    pub(crate) workspace: &'ctx Workspace,
    /// Scope of the command.
    pub(crate) scope: Scope,
}

/// Information needed to execute a command.
///
/// Importantly, as commands have different purposes, different command scopes
/// exist to cater for each kind of command. This means the data available in a
/// command context differs per scope, to accurately reflect what is available.
#[derive(Debug)]
pub struct CmdCtxView<'view, O, Scope> {
    /// Output endpoint to return values / errors, and write progress
    /// information to.
    ///
    /// See [`OutputWrite`].
    ///
    /// [`OutputWrite`]: peace_rt_model_core::OutputWrite
    pub output: &'view mut O,
    /// Workspace that the `peace` tool runs in.
    pub workspace: &'view Workspace,
    /// Scope of the command.
    pub scope: &'view mut Scope,
}

impl<'ctx, O, Scope> CmdCtx<'ctx, O, Scope> {
    /// Returns a view struct of this command context.
    ///
    /// This allows the output and scope data to be borrowed concurrently.
    pub fn view(&mut self) -> CmdCtxView<'_, O, Scope> {
        let Self {
            output,
            workspace,
            scope,
        } = self;

        CmdCtxView {
            output,
            workspace,
            scope,
        }
    }

    /// Returns a reference to the output.
    pub fn output(&self) -> &O {
        self.output
    }

    /// Returns a mutable reference to the output.
    pub fn output_mut(&mut self) -> &mut O {
        self.output
    }

    /// Returns the workspace that the `peace` tool runs in.
    pub fn workspace(&self) -> &Workspace {
        self.workspace
    }

    /// Returns the scope of the command.
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    /// Returns a mutable reference to the scope of the command.
    pub fn scope_mut(&mut self) -> &mut Scope {
        &mut self.scope
    }

    /// Returns a reference to the workspace directory.
    pub fn workspace_dir(&self) -> &WorkspaceDir {
        self.workspace.dirs().workspace_dir()
    }

    /// Returns a reference to the `.peace` directory.
    pub fn peace_dir(&self) -> &PeaceDir {
        self.workspace.dirs().peace_dir()
    }

    /// Returns a reference to the `.peace/$app` directory.
    pub fn peace_app_dir(&self) -> &PeaceAppDir {
        self.workspace.dirs().peace_app_dir()
    }
}

impl<'ctx, O> CmdCtx<'ctx, O, ()> {
    /// Returns a `CmdCtxBuilder` for a single profile and no flow.
    pub fn builder_no_profile_no_flow<E>(
        output: &'ctx mut O,
        workspace: &'ctx Workspace,
    ) -> CmdCtxBuilder<
        'ctx,
        O,
        NoProfileNoFlowBuilder<
            E,
            ParamsKeysImpl<KeyUnknown, KeyUnknown, KeyUnknown>,
            WorkspaceParamsNone,
        >,
    > {
        CmdCtxBuilder::no_profile_no_flow(output, workspace)
    }

    /// Returns a `CmdCtxBuilder` for multiple profiles and no flow.
    pub fn builder_multi_profile_no_flow<E>(
        output: &'ctx mut O,
        workspace: &'ctx Workspace,
    ) -> CmdCtxBuilder<
        'ctx,
        O,
        MultiProfileNoFlowBuilder<
            E,
            ProfileNotSelected,
            ParamsKeysImpl<KeyUnknown, KeyUnknown, KeyUnknown>,
            WorkspaceParamsNone,
            ProfileParamsNone,
        >,
    > {
        CmdCtxBuilder::multi_profile_no_flow(output, workspace)
    }

    /// Returns a `CmdCtxBuilder` for multiple profiles and one flow.
    pub fn builder_multi_profile_single_flow<E>(
        output: &'ctx mut O,
        workspace: &'ctx Workspace,
    ) -> CmdCtxBuilder<
        'ctx,
        O,
        MultiProfileSingleFlowBuilder<
            E,
            ProfileNotSelected,
            FlowNotSelected,
            ParamsKeysImpl<KeyUnknown, KeyUnknown, KeyUnknown>,
            WorkspaceParamsNone,
            ProfileParamsNone,
            FlowParamsNone,
        >,
    > {
        CmdCtxBuilder::multi_profile_single_flow(output, workspace)
    }

    /// Returns a `CmdCtxBuilder` for a single profile and flow.
    pub fn builder_single_profile_no_flow<E>(
        output: &'ctx mut O,
        workspace: &'ctx Workspace,
    ) -> CmdCtxBuilder<
        'ctx,
        O,
        SingleProfileNoFlowBuilder<
            E,
            ProfileNotSelected,
            ParamsKeysImpl<KeyUnknown, KeyUnknown, KeyUnknown>,
            WorkspaceParamsNone,
            ProfileParamsNone,
        >,
    > {
        CmdCtxBuilder::single_profile_no_flow(output, workspace)
    }

    /// Returns a `CmdCtxBuilder` for a single profile and flow.
    pub fn builder_single_profile_single_flow<E>(
        output: &'ctx mut O,
        workspace: &'ctx Workspace,
    ) -> CmdCtxBuilder<
        'ctx,
        O,
        SingleProfileSingleFlowBuilder<
            E,
            ProfileNotSelected,
            FlowNotSelected,
            ParamsKeysImpl<KeyUnknown, KeyUnknown, KeyUnknown>,
            WorkspaceParamsNone,
            ProfileParamsNone,
            FlowParamsNone,
        >,
    > {
        CmdCtxBuilder::single_profile_single_flow(output, workspace)
    }
}

impl<'ctx, E, O, PKeys, ResTs0> CmdCtx<'ctx, O, SingleProfileSingleFlow<E, PKeys, ResTs0>>
where
    PKeys: ParamsKeys + 'static,
{
    /// Updates `resources` to a different type state based on the given
    /// function.
    pub fn resources_update<ResTs1, F>(
        self,
        f: F,
    ) -> CmdCtx<'ctx, O, SingleProfileSingleFlow<E, PKeys, ResTs1>>
    where
        F: FnOnce(Resources<ResTs0>) -> Resources<ResTs1>,
    {
        let CmdCtx {
            output,
            workspace,
            scope,
        } = self;

        let scope = scope.resources_update(f);

        CmdCtx {
            output,
            workspace,
            scope,
        }
    }
}

impl<'ctx, O, Scope> Deref for CmdCtx<'ctx, O, Scope> {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl<'ctx, O, Scope> DerefMut for CmdCtx<'ctx, O, Scope> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}
