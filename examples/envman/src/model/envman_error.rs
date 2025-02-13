#[cfg(feature = "error_reporting")]
use peace::miette;

use peace::{
    cfg::{AppName, Profile},
    rt_model::fn_graph::{Edge, WouldCycle},
};

/// Error while managing a web application.
#[cfg_attr(feature = "error_reporting", derive(peace::miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum EnvManError {
    /// Failed to construct web application download URL.
    #[error("Failed to construct web application download URL.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(envman::envman_url_build),
            help("If the URL is valid, this may be a bug in the example, or the `url` library.")
        )
    )]
    EnvManUrlBuild {
        /// Computed URL that is attempted to be parsed.
        #[cfg_attr(feature = "error_reporting", source_code)]
        url_candidate: String,
        /// URL parse error.
        #[source]
        error: url::ParseError,
    },
    /// Failed to parse environment type.
    #[error("Failed to parse environment type.")]
    EnvTypeParseError(
        #[source]
        #[from]
        crate::model::EnvTypeParseError,
    ),

    /// User tried to switch to a profile that doesn't exist.
    #[error("Profile to switch to does not exist.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(envman::profile_switch_to_non_existent),
            help(
                "The `{profile_to_switch_to}` profile does not exist.\n\
                You can create it by passing the `--create --type development` parameters\n\
                or run `{app_name} profile list` to see profiles you can switch to."
            )
        )
    )]
    ProfileSwitchToNonExistent {
        /// The profile that the user tried to switch to.
        profile_to_switch_to: Profile,
        /// Name of this app.
        app_name: AppName,
    },

    /// User tried to create a profile that already exists.
    #[error("Profile to create already exists.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(envman::profile_to_create_exists),
            help(
                "The `{profile_to_create}` profile already exists.\n\
                You may switch to the profile using `{app_name} switch {profile_to_create}`\n\
                or create a profile with a different name."
            )
        )
    )]
    ProfileToCreateExists {
        /// The profile that the user tried to create.
        profile_to_create: Profile,
        /// Name of this app.
        app_name: AppName,
    },

    // === Item Spec errors === //
    /// A `FileDownload` item spec error occurred.
    #[error("A `FileDownload` item spec error occurred.")]
    PeaceItemSpecFileDownload(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace_item_specs::file_download::FileDownloadError,
    ),
    /// A `TarX` item spec error occurred.
    #[error("A `TarX` item spec error occurred.")]
    PeaceItemSpecTarX(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace_item_specs::tar_x::TarXError,
    ),
    /// An `InstanceProfile` item spec error occurred.
    #[error("An `InstanceProfile` item spec error occurred.")]
    InstanceProfileItemSpec(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::item_specs::peace_aws_instance_profile::InstanceProfileError,
    ),
    /// An `IamPolicy` item spec error occurred.
    #[error("An `IamPolicy` item spec error occurred.")]
    IamPolicyItemSpec(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::item_specs::peace_aws_iam_policy::IamPolicyError,
    ),
    /// An `IamRole` item spec error occurred.
    #[error("An `IamRole` item spec error occurred.")]
    IamRoleItemSpec(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::item_specs::peace_aws_iam_role::IamRoleError,
    ),
    /// An `S3Bucket` item spec error occurred.
    #[error("An `S3Bucket` item spec error occurred.")]
    S3BucketItemSpec(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::item_specs::peace_aws_s3_bucket::S3BucketError,
    ),
    /// An `S3Object` item spec error occurred.
    #[error("An `S3Object` item spec error occurred.")]
    S3ObjectItemSpec(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::item_specs::peace_aws_s3_object::S3ObjectError,
    ),

    // === Framework errors === //
    /// A `peace` runtime error occurred.
    #[error("A `peace` runtime error occurred.")]
    PeaceRtError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace::rt_model::Error,
    ),

    /// A graph `WouldCycle` error occurred.
    #[error("A `peace` runtime error occurred.")]
    WouldCycleError(
        #[source]
        #[from]
        WouldCycle<Edge>,
    ),

    // === Scaffolding errors === //
    #[error("Failed to initialize tokio runtime.")]
    TokioRuntimeInit(#[source] std::io::Error),
}
