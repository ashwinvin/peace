use std::fmt;

use serde::{Deserialize, Serialize};

use crate::item_specs::peace_aws_iam_role::model::ManagedPolicyAttachment;

/// Diff between current (dest) and desired (src) state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum IamRoleStateDiff {
    /// Role would be added.
    Added,
    /// Role would be removed.
    Removed,
    /// The managed policy attached to the role has changed.
    ManagedPolicyAttachmentModified {
        /// Current state of the managed policy attachment.
        managed_policy_attachment_current: ManagedPolicyAttachment,
        /// Desired state of the managed policy attachment.
        managed_policy_attachment_desired: ManagedPolicyAttachment,
    },
    /// Role would be replaced.
    ///
    /// AWS doesn't support modifying a role in place, so this item spec must be
    /// cleaned and re-created.
    NameOrPathModified {
        /// Whether the name has been changed.
        name_diff: Option<(String, String)>,
        /// Whether the path has been changed.
        path_diff: Option<(String, String)>,
    },
    /// Role exists and is up to date.
    InSyncExists,
    /// Role does not exist, which is desired.
    InSyncDoesNotExist,
}

impl fmt::Display for IamRoleStateDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IamRoleStateDiff::Added => {
                write!(f, "will be created.")
            }
            IamRoleStateDiff::Removed => {
                write!(f, "will be removed.")
            }
            IamRoleStateDiff::ManagedPolicyAttachmentModified {
                managed_policy_attachment_current,
                managed_policy_attachment_desired,
            } => {
                if managed_policy_attachment_current.arn()
                    != managed_policy_attachment_desired.arn()
                {
                    write!(f, "Managed policy attachment will be replaced.")
                } else {
                    match (
                        managed_policy_attachment_current.attached(),
                        managed_policy_attachment_desired.attached(),
                    ) {
                        (false, false) | (true, true) => unreachable!(
                            "If the attached managed policy ARNs are the same, then the attached state must be different."
                        ),
                        (true, false) => write!(f, "Managed policy will be detached."),
                        (false, true) => write!(f, "Managed policy will be attached."),
                    }
                }
            }
            IamRoleStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => match (name_diff, path_diff) {
                (None, None) => {
                    unreachable!(
                        "Modified is only valid when either name or path has changed.\n\
                        This is a bug."
                    )
                }
                (None, Some((path_current, path_desired))) => write!(
                    f,
                    "path modified from {path_current} to {path_desired}. ⚠️ This modification requires deletion and recreation."
                ),
                (Some((name_current, name_desired)), None) => write!(
                    f,
                    "name modified from {name_current} to {name_desired}. ⚠️ This modification requires deletion and recreation."
                ),

                (Some((name_current, name_desired)), Some((path_current, path_desired))) => write!(
                    f,
                    "modified from {path_current}{name_current} to {path_desired}{name_desired}. ⚠️ This modification requires deletion and recreation."
                ),
            },
            IamRoleStateDiff::InSyncExists => {
                write!(f, "exists and is up to date.")
            }
            IamRoleStateDiff::InSyncDoesNotExist => {
                write!(f, "does not exist as intended.")
            }
        }
    }
}
