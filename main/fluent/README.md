# reaper-fluent

This is an experiment.

Goals:

- Provide a light-weight fluent API on top of `reaper-medium`.
- Do just that, nothing else. `reaper-high` suffers from offering too much.
- Stay relatively close to `reaper-medium` in terms of possibilities, parameters and types. Reuse medium types
  whenever possible.
- Design the API in a way that makes it hard to crash REAPER. But instead of checking in each function call
  if the object in question is still valid (as done in `reaper-high`), try to leverage Rust's borrow checking,
  trait system and lifetime mechanics to minimize the amount of necessary runtime checks.
- For each kind of object, distinguish between at least:
  - Descriptor
    - Describes the unique address of the object, so it can be restored at any time
    - Doesn't provide any methods related to querying/modifying the object
    - Standalone value (zero borrows, zero sharing, no pointers if possible)
    - If possible, make it `Copy`
    - Can be created and used very freely from anywhere, can be stored anywhere
    - Can be fallibly turned into a resolved object by passing it to the REAPER object model
    - Example: `TrackDesc` would contain a project handle and the unique track GUID
    - Use case: Storing a long-lived reference to an object. Whenever you care about restoring precisely what you
      have stored.
  - Resolved object
    - A zero-cost wrapper around the native REAPER object pointer
    - Contains all the object's methods and doesn't check validity in them (safe due to lifetime constraints)
    - Can't be stored in structs because very ephemeral in nature
    - Can only be obtained by navigating the REAPER object model
    - Example: `Track` would just contain a `MediaTrack` pointer and a lifetime reference to a `Project` object
    - Use case: Querying and manipulating an object
  - Pointer object
    - A zero-cost wrapper around the native REAPER object pointer
    - Doesn't provide any methods related to querying/modifying the object
    - Can be created and used very freely from anywhere, can be stored anywhere
    - Can be fallibly turned into a resolved object (fast because just a pointer validity check)
    - Example: `TrackPtr` would contain a `MediaTrack` pointer
    - Use case: Storing a somewhat longer-lived reference to an object. Whenever you care more about performance
      than restoring precisely what you have stored (in theory, the pointer could point to another object after a while)
- For tracks, distinguish between master track and normal track. They behave quite differently. This enables
  for example to make `name()` return a simple string instead of an optional string (for normal tracks).