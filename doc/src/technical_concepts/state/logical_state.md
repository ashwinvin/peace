# Logical State

Logical state is the part of an item that is:

* Implementor / user definable
* Controllable by automation
* Deterministic

## Uses

There are three uses of logical state:

1. Representing current state.
2. Representing desired state.
3. Computing state difference.

```rust ,ignore
let params = data.params();

// `dest` references the actual item that is being managed.
// e.g. calculate content hash of actual file being written to.
let state_current = params.dest().read();

// `src` references the *specification* of what the item is intended to be.
// e.g. retrieve content hash from a source file.
let state_desired = params.src().read();

// We can only compute the `diff` when both `src` and `dest` are available.
let state_diff = state_desired - state_current;
```

## Discovery Constraints

In an item spec's parameters, there must be the following categories of information:

* `src`: information of what the item should be, or where to look up that information.

    Thus, `src` is a reference to where to look up `state_desired`.

* `dest`: reference to where the actual item should be.

    `dest` is a reference to where to push `state_current`.

Both `src` and `dest` may reference resources that are ensured by predecessor item specs. Meaning sometimes `state_desired` and `state_current` cannot be discovered because they rely on the predecessors' completions.

### Examples

* A list of files in a zip file cannot be read, if the zip file is not downloaded.
* A file on a server cannot be read, if the server doesn't exist.
* A server cannot have a domain name assigned to it, if the server doesn't exist.

### Implications

* If `dest` is not available, then `state_current` may simply be "does not exist".
* If `src` is not available, and we want to show `state_desired` that is not just "we can't look it up", then `src` must be defined in terms of something readable during discovery.
* If that is not possible, or is too expensive, then one or more of the following has to be chosen:

    1. `StateDesiredFnSpec`s have to always cater for `src` not being available.

        It incurs mental effort to always cater for `src` not being available &ndash; i.e. implementing an item spec would need knowledge beyond itself.

    2. the `peace` framework defaults to not running `state_current_fn_spec` for items that have a logical dependency on things that `ApplyOpSpec::check` returns `ExecRequired`

        For this to work, when `StateCurrentFnSpec::try_exec` is requested, `peace` will:

        1. For each non-parent item, run `StateCurrentFnSpec`, `StateDesiredFnSpec`, `StateDiffFnSpec`, and `ApplyOpSpec::check`.
        2. If `ApplyOpSpec::check` returns `OpCheckStatus::ExecNotRequired`, then successor items can be processed as well.

    3. `StateCurrentFnSpec` could return `Result<Option<Status>, E>`:

        + `Ok(None)`: State cannot be discovered, likely because predecessor hasn't run
        + `Ok(Some(State<_, _>))`: State cannot be discovered.
        + `Err(E)`: An error happened when discovering state.

            May be difficult to distinguish some cases from `Ok(None)`, e.g. failed to connect to server, is it because the server doesn't exist, or because the address is incorrect.

            Should we have two `StateCurrentFnSpec`s? Or pass in whether it's being called from `Discover` vs `Ensure` &ndash; i.e. some information that says "err when failing to connect because the predecessor has been ensured".

        <!--  -->

    Option 2 may be something we have to do anyway &ndash; we will not be able to provide current state to run `ApplyOpSpec::exec` for successors for the same reason.

    Option 3 may coexist with option 2.

    **Note:** State discovery may be expensive, so it is important to be able to show a saved copy of what is discovered.
