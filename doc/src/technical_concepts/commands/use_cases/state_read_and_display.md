# State Read and Display

> This is like `git show`.

This kind of command is intended to display state that has previously been discovered.

Suitable scopes for this command are:

* `SingleProfileSingleFlow`: For reading state for one profile.
* `MultiProfileSingleFlow`: For reading state for multiple profiles.


## Command Creation

To create this command:

1. When building the command context:

    - Provide the profile.
    - Provide the flow ID.

2. Call one of the state read commands depending on the intended use:

    These will store the discovered states under the corresponding `$profile/$flow_id` directory as `states_saved.yaml` or `states_desired.yaml`.

    - `StatesSavedReadCmd`: For current states to be discovered.
    - `StatesDesiredReadCmd`: For desired states to be discovered.

3. Call the relevant state display command(s):

    - `StatesSavedDisplayCmd`: For current states to be displayed.
    - `StatesDesiredDisplayCmd`: For desired states to be displayed.
