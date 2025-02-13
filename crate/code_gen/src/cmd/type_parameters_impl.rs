use syn::{parse_quote, punctuated::Punctuated, GenericArgument, Token};

use crate::cmd::{FlowCount, ProfileCount, Scope};

use super::ParamsScope;

/// Appends profile / flow ID selection type parameters if applicable to the
/// given scope.
pub fn profile_and_flow_selection_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
) {
    match scope.profile_count() {
        ProfileCount::None => {}
        ProfileCount::One | ProfileCount::Multiple => {
            type_params.push(parse_quote!(ProfileSelection));
        }
    }
    if scope.flow_count() == FlowCount::One {
        type_params.push(parse_quote!(FlowSelection));
    }
}

/// Appends workspace / profile / flow params selection type parameters if
/// applicable to the given scope.
pub fn params_selection_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
) {
    // Always collect PKeys.
    type_params.push(parse_quote!(PKeys));

    // Workspace params are supported by all scopes.
    type_params.push(parse_quote!(WorkspaceParamsSelection));

    if scope.profile_params_supported() {
        type_params.push(parse_quote!(ProfileParamsSelection));
    }

    if scope.flow_params_supported() {
        type_params.push(parse_quote!(FlowParamsSelection));
    }
}

/// Appends the type parameters for params selection for the `impl`.
///
/// For the `Workspace` params scope, the following are generated, if
/// applicable to the current command context builder scope:
///
/// * `ProfileParamsSelection`: To retain any existing profile params selection.
/// * `FlowParamsSelection`: To retain any existing flow params selection.
/// * `ProfileParamsKMaybe`: To retain the key for existing profile params
///   selection.
/// * `FlowParamsKMaybe`: To retain the key for existing flow params selection.
pub fn params_selection_maybe_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
    params_scope: ParamsScope,
    params_key_known: bool,
) {
    match params_scope {
        ParamsScope::Workspace => {
            if params_key_known {
                type_params.push(parse_quote!(WorkspaceParamsK));
            }

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsSelection));
                type_params.push(parse_quote!(ProfileParamsKMaybe));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsSelection));
                type_params.push(parse_quote!(FlowParamsKMaybe));
            }
        }
        ParamsScope::Profile => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsSelection));
            type_params.push(parse_quote!(WorkspaceParamsKMaybe));

            if scope.profile_params_supported() && params_key_known {
                type_params.push(parse_quote!(ProfileParamsK));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsSelection));
                type_params.push(parse_quote!(FlowParamsKMaybe));
            }
        }
        ParamsScope::Flow => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsSelection));
            type_params.push(parse_quote!(WorkspaceParamsKMaybe));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsSelection));
                type_params.push(parse_quote!(ProfileParamsKMaybe));
            }

            if scope.flow_params_supported() && params_key_known {
                type_params.push(parse_quote!(FlowParamsK));
            }
        }
    }
}

/// Appends the type parameters for params selection for the `ScopeBuilder` in
/// the `impl`.
///
/// For the `Workspace` params scope, the following are generated, if
/// applicable to the current command context builder scope:
///
/// * `WorkspaceParamsNone`: Indicates that the incoming that params selection
///   is none.
/// * `ProfileParamsSelection`: To retain any existing profile params selection.
/// * `FlowParamsSelection`: To retain any existing flow params selection.
pub fn params_selection_none_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
    params_scope: ParamsScope,
) {
    let impl_params_key_unknown_params = {
        let mut type_params = Punctuated::<GenericArgument, Token![,]>::new();
        params_key_unknown_push(&mut type_params, scope, params_scope);
        type_params
    };

    type_params.push(parse_quote! {
        peace_rt_model::params::ParamsKeysImpl<
            // KeyUnknown, ProfileParamsKMaybe, FlowParamsKMaybe
            #impl_params_key_unknown_params
        >
    });

    match params_scope {
        ParamsScope::Workspace => {
            type_params.push(parse_quote!(
                crate::scopes::type_params::WorkspaceParamsNone
            ));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsSelection));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsSelection));
            }
        }
        ParamsScope::Profile => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsSelection));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(crate::scopes::type_params::ProfileParamsNone));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsSelection));
            }
        }
        ParamsScope::Flow => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsSelection));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsSelection));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(crate::scopes::type_params::FlowParamsNone));
            }
        }
    }
}

/// Appends the type parameters for params selection for the `ScopeBuilder` in
/// the `impl`.
///
/// For the `Workspace` params scope, the following are generated, if
/// applicable to the current command context builder scope:
///
/// * `WorkspaceParamsSome<WorkspaceParamsK>`: Indicates that the incoming
///   params selection is none.
/// * `ProfileParamsSelection`: To retain any existing profile params selection.
/// * `FlowParamsSelection`: To retain any existing flow params selection.
pub fn params_selection_some_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
    params_scope: ParamsScope,
) {
    let impl_params_key_known_params = {
        let mut type_params = Punctuated::<GenericArgument, Token![,]>::new();
        params_key_known_push(&mut type_params, scope, params_scope);
        type_params
    };

    type_params.push(parse_quote! {
        peace_rt_model::params::ParamsKeysImpl<
            // KeyKnown<WorkspaceParamsK>, ProfileParamsKMaybe, FlowParamsKMaybe
            #impl_params_key_known_params
        >
    });

    match params_scope {
        ParamsScope::Workspace => {
            type_params.push(parse_quote!(
                crate::scopes::type_params::WorkspaceParamsSome<WorkspaceParamsK>
            ));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsSelection));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsSelection));
            }
        }
        ParamsScope::Profile => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsSelection));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(
                    crate::scopes::type_params::ProfileParamsSome<ProfileParamsK>
                ));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsSelection));
            }
        }
        ParamsScope::Flow => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsSelection));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsSelection));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(
                    crate::scopes::type_params::FlowParamsSome<FlowParamsK>
                ));
            }
        }
    }
}

/// Appends the type parameters for params selection for the `ScopeBuilder` in
/// the `impl`.
///
/// For the `Workspace` params scope, the following are generated, if
/// applicable to the current command context builder scope:
///
/// * `KeyUnknown`: Indicates that the incoming params key is known.
/// * `ProfileParamsKMaybe`: To retain any existing profile params key.
/// * `FlowParamsKMaybe`: To retain any existing flow params key.
pub fn params_key_unknown_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
    params_scope: ParamsScope,
) {
    match params_scope {
        ParamsScope::Workspace => {
            type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }
        }
        ParamsScope::Profile => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsKMaybe));

            type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }
        }
        ParamsScope::Flow => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsKMaybe));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }

            type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
        }
    }
}

/// Appends the type parameters for params selection for the `ScopeBuilder` in
/// the `impl`.
///
/// For the `Workspace` params scope, the following are generated, if
/// applicable to the current command context builder scope:
///
/// * `KeyKnown<WorkspaceParamsK>`: Indicates that the outgoing params key is
///   known.
/// * `ProfileParamsKMaybe`: To retain any existing profile params key.
/// * `FlowParamsKMaybe`: To retain any existing flow params key.
pub fn params_key_known_push(
    type_params: &mut Punctuated<GenericArgument, Token![,]>,
    scope: Scope,
    params_scope: ParamsScope,
) {
    match params_scope {
        ParamsScope::Workspace => {
            type_params.push(parse_quote!(
                peace_rt_model::params::KeyKnown<WorkspaceParamsK>
            ));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }
        }
        ParamsScope::Profile => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsKMaybe));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(
                    peace_rt_model::params::KeyKnown<ProfileParamsK>
                ));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(FlowParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }
        }
        ParamsScope::Flow => {
            // Workspace params are supported by all scopes.
            type_params.push(parse_quote!(WorkspaceParamsKMaybe));

            if scope.profile_params_supported() {
                type_params.push(parse_quote!(ProfileParamsKMaybe));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }

            if scope.flow_params_supported() {
                type_params.push(parse_quote!(peace_rt_model::params::KeyKnown<FlowParamsK>));
            } else {
                type_params.push(parse_quote!(peace_rt_model::params::KeyUnknown));
            }
        }
    }
}
