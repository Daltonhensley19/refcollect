use gc::{DummyObject, MarkandSweepGC};

// Amount of roots for initializing the `MarkandSweepGC`.
const INITIAL_ROOT_AMOUNT: usize = 2;

fn main() {
    // Initalize the garbage collector which uses the mark and sweep algorithm.
    let mut gc_arena = MarkandSweepGC::new_with_test_dummy_roots(INITIAL_ROOT_AMOUNT);

    // Create distinct `DummyObject` pointers which will be associated with the roots
    // of the `gc_arena`.
    let ptrs: Vec<*mut DummyObject> = (0..INITIAL_ROOT_AMOUNT)
        .map(|_| DummyObject::new_on_heap())
        .collect();

    // Associate each pointer in `ptrs` with the intial roots of `gc_arena`.
    for ptr in ptrs {
        gc_arena.refernce_dummy_at_to(0, ptr);
    }

    // Test `sweep()` which will do nothing since no roots are marked
    gc_arena.sweep();

    // Mark root `DummyObject` in the `gc_arena` as unreachable.
    gc_arena.mark_unreachable(1, 0);

    // Show that the root along the path `root -> ... -> NULL` is not reachable
    gc_arena.display_root_trail_values(0);

    // Run another `sweep()` to clear the path `root -> ... -> NULL` being automatically freed
    gc_arena.sweep();

    gc_arena.display_root_trail_values(0);

    // NOTE: `gc_arena` uses RAII to automatically clean up all internal roots and their
    // references to `DummyObject`'s when `gc_arena` goes out of scope. Use `gc_arena.leak()` to
    // prevent this behavior.
}
