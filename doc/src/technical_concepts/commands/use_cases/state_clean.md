# State Clean

> This is like `rm -rf`.

This kind of command applies the clean state over the current state.

This generally requires what is saved in `states_saved.yaml` to match the newly discovered current state.

The only suitable scope for this command is `SingleProfileSingleFlow`.


## Command Creation

To create this command:

1. When building the command context:

    - Provide the profile.
    - Provide the flow ID.

2. Call the `CleanCmd`.

    This will call `CleanOpSpec::exec` for each item spec, beginning from the last item spec, until the first item spec.
