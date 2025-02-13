use std::path::PathBuf;

use peace_core::FlowId;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub use self::native_error::NativeError;

        mod native_error;
    } else {
        pub use self::web_error::WebError;

        mod web_error;
    }
}

/// Peace runtime errors.
#[cfg_attr(feature = "error_reporting", derive(miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to serialize error.
    #[error("Failed to serialize error.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::error_serialize))
    )]
    ErrorSerialize(#[source] serde_yaml::Error),

    /// Failed to serialize a presentable type.
    #[error("Failed to serialize a presentable type.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::presentable_serialize))
    )]
    PresentableSerialize(#[source] serde_yaml::Error),

    /// Failed to serialize progress update.
    #[error("Failed to serialize progress update.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::progress_update_serialize))
    )]
    ProgressUpdateSerialize(#[source] serde_yaml::Error),
    /// Failed to serialize progress update as JSON.
    #[cfg(feature = "output_json")]
    #[error("Failed to serialize progress update.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::progress_update_serialize_json))
    )]
    ProgressUpdateSerializeJson(#[source] serde_json::Error),

    /// Failed to deserialize states.
    #[error("Failed to deserialize states for flow: `{flow_id}`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::states_deserialize),
            help(
                "Make sure that all commands using the `{flow_id}` flow, also use the same item spec graph.\n\
                This is because all ItemSpecs are used to deserialize state.\n\
                \n\
                If the item spec graph is different, it may make sense to use a different flow ID."
            )
        )
    )]
    StatesDeserialize {
        /// Flow ID whose states are being deserialized.
        flow_id: FlowId,
        /// Source text to be deserialized.
        #[cfg(feature = "error_reporting")]
        #[source_code]
        states_file_source: miette::NamedSource,
        /// Offset within the source text that the error occurred.
        #[cfg(feature = "error_reporting")]
        #[label("{}", error_message)]
        error_span: Option<miette::SourceOffset>,
        /// Message explaining the error.
        #[cfg(feature = "error_reporting")]
        error_message: String,
        /// Offset within the source text surrounding the error.
        #[cfg(feature = "error_reporting")]
        #[label]
        context_span: Option<miette::SourceOffset>,
        /// Underlying error.
        #[source]
        error: serde_yaml::Error,
    },

    /// Failed to serialize states.
    #[error("Failed to serialize states.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::states_serialize))
    )]
    StatesSerialize(#[source] serde_yaml::Error),

    /// Current states have not been discovered.
    ///
    /// This is returned when `StatesSavedFile` is attempted to be
    /// deserialized but does not exist.
    #[error("Current states have not been discovered.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::states_current_discover_required),
            help("Ensure that `StatesDiscoverCmd` or `StatesCurrentDiscoverCmd` has been called.")
        )
    )]
    StatesCurrentDiscoverRequired,

    /// Desired states have not been written to disk.
    ///
    /// This is returned when `StatesDesiredFile` is attempted to be
    /// deserialized but does not exist.
    #[error("Desired states have not been written to disk.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::states_desired_discover_required),
            help("Ensure that `StatesDiscoverCmd` or `StatesDesiredDiscoverCmd` has been called.")
        )
    )]
    StatesDesiredDiscoverRequired,

    /// Failed to serialize state diffs.
    #[error("Failed to serialize state diffs.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::state_diffs_serialize))
    )]
    StateDiffsSerialize(#[source] serde_yaml::Error),

    /// Failed to serialize error as JSON.
    #[cfg(feature = "output_json")]
    #[error("Failed to serialize error as JSON.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::error_serialize_json))
    )]
    ErrorSerializeJson(#[source] serde_json::Error),

    /// Failed to serialize states as JSON.
    #[cfg(feature = "output_json")]
    #[error("Failed to serialize states as JSON.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::states_current_serialize_json))
    )]
    StatesSerializeJson(#[source] serde_json::Error),

    /// Failed to serialize state diffs as JSON.
    #[cfg(feature = "output_json")]
    #[error("Failed to serialize state diffs as JSON.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::state_diffs_serialize_json))
    )]
    StateDiffsSerializeJson(#[source] serde_json::Error),

    /// Failed to serialize workspace init params.
    #[error("Failed to serialize workspace init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_init_params_serialize))
    )]
    WorkspaceParamsSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize workspace init params.
    #[error("Failed to deserialize workspace init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_init_params_deserialize))
    )]
    WorkspaceParamsDeserialize(#[source] serde_yaml::Error),

    /// Workspace params does not exist, so cannot look up `Profile`.
    #[error("Workspace params does not exist, so cannot look up `Profile`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_params_none_for_profile))
    )]
    WorkspaceParamsNoneForProfile,

    /// Workspace param for `Profile` does not exist.
    #[error("Workspace param for `Profile` does not exist.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_params_profile_none))
    )]
    WorkspaceParamsProfileNone,

    /// Failed to serialize profile init params.
    #[error("Failed to serialize profile init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::profile_init_params_serialize))
    )]
    ProfileParamsSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize profile init params.
    #[error("Failed to deserialize profile init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::profile_init_params_deserialize))
    )]
    ProfileParamsDeserialize(#[source] serde_yaml::Error),

    /// Failed to serialize flow init params.
    #[error("Failed to serialize flow init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::flow_init_params_serialize))
    )]
    FlowParamsSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize flow init params.
    #[error("Failed to deserialize flow init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::flow_init_params_deserialize))
    )]
    FlowParamsDeserialize(#[source] serde_yaml::Error),

    /// Item does not exist in storage.
    #[error("Item does not exist in storage: `{}`.", path.display())]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::item_not_exists))
    )]
    ItemNotExists {
        /// Path to the file.
        path: PathBuf,
    },

    /// Native application error occurred.
    #[error("Native application error occurred.")]
    #[cfg(not(target_arch = "wasm32"))]
    Native(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        NativeError,
    ),

    /// Web application error occurred.
    #[error("Web application error occurred.")]
    #[cfg(target_arch = "wasm32")]
    Web(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        WebError,
    ),
}
