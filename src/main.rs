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

    // gc_arena.mark_unreachable(0, 1);
    gc_arena.display_root_trail_values(0);
}
