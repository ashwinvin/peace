use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use peace_core::Profile;
use peace_rt_model::params::ProfileParams;
use serde::{de::DeserializeOwned, Serialize};

/// The application does not use any profile parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileParamsNone;

/// The application has profile parameters.
#[derive(Debug)]
pub struct ProfileParamsSome<ProfileParamsK>(pub(crate) ProfileParams<ProfileParamsK>)
where
    ProfileParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static;

/// The application has profile parameters from multiple profiles.
#[derive(Debug)]
pub struct ProfileParamsSomeMulti<ProfileParamsK>(
    pub(crate) BTreeMap<Profile, ProfileParams<ProfileParamsK>>,
)
where
    ProfileParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static;
