use gc::{DummyObject, MarkandSweepGC};

fn main() {
    const DUMMY_AMOUNT: usize = 2;
    let mut gc_arena = MarkandSweepGC::new_spawn_test_dummies(DUMMY_AMOUNT);

    let ptr = DummyObject::new_on_heap();

    let mut ptrs = vec![];
    for _ in 0..DUMMY_AMOUNT {
        ptrs.push(DummyObject::new_on_heap());
    }

    // unsafe {
    //     dbg!(&*(ptr));
    // }

    for i in 0..DUMMY_AMOUNT {
        gc_arena.refernce_dummy_at_to(0, ptrs[i]);
    }

    // gc_arena.mark_unreachable(0, 1);
    gc_arena.display_root_trail_values(0);

    //
    // {
    //     let d3 = DummyObject::new_on_heap();
    //     let d4 = DummyObject::new_on_heap();
    //     let d5 = DummyObject::new_on_heap();
    //
    //     let root_idx = 0;
    //     // println!("[Before adding reference at index {root_idx}]");
    //     // println!("{gc_arena:#?}");
    //     gc_arena.refernce_dummy_at_to(root_idx, d3);
    //     gc_arena.refernce_dummy_at_to(root_idx, d4);
    //     gc_arena.refernce_dummy_at_to(root_idx, d5);
    //     // println!("\n\n\n[After adding reference at index {root_idx}]");
    //     println!("{gc_arena:#?}");
    //
    //     // gc_arena.display_root_trail_addresses(root_idx);
    // }

    // gc_arena.display_root_trail_values(0);
}
