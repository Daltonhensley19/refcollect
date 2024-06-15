use rand::{thread_rng, Rng};

#[derive(Debug, Clone)]
struct DummyObject {
    marked: bool,
    data1: i32,
    data2: f32,
    next: *mut DummyObject,
}

impl DummyObject {
    fn new() -> Self {
        let mut rng = thread_rng();
        Self {
            marked: false,
            data1: rng.gen_range(0..100),
            data2: rng.gen_range(0.0..100.0),
            next: std::ptr::null_mut(),
        }
    }

    fn new_on_heap() -> *mut DummyObject {
        let layout = std::alloc::Layout::new::<DummyObject>();

        // SAFETY: `ptr` must be be assigned a value of `DummyObject`
        // in order for it to be valid.
        let ptr = unsafe { std::alloc::alloc(layout) };

        // SAFETY: Users of `ptr` must ensure that `ptr` is not null.
        unsafe {
            *(ptr as *mut DummyObject) = DummyObject::new();
        }

        ptr as *mut DummyObject
    }

    fn reference_to(&mut self, dummy: *mut DummyObject) {
        if self.next.is_null() {
            self.next = dummy;
        } else {
            // Find the end of the list
            let mut ptr = self.next;

            // SAFETY: `ptr.next` is not null here, so we can safely
            // dereference it inside the while loop.
            unsafe {
                while !(*ptr).next.is_null() {
                    ptr = (*ptr).next;
                }
            }

            // SAFETY: `ptr.next` is not null here, so we can safely
            // dereference it.
            unsafe {
                (*ptr).next = dummy;
            }
        }
    }

    fn free(mut object: *mut DummyObject) {
        assert!(
            !object.is_null(),
            "Memory deallocation failed due to trying to free a null pointer!"
        );
        let layout = std::alloc::Layout::new::<DummyObject>();

        // SAFETY: `object` must refer to a chuck of memory that is live
        // and allocated/aligned to a `layout` of `DummyObject`.
        unsafe { std::alloc::dealloc(object as *mut u8, layout) };
    }
}

/// This is a memory arena that keeps track of the references to roots of `DummyObject`
#[derive(Debug, Default)]
struct MarkandSweepGC {
    roots: Vec<*mut DummyObject>,
}

// RAII for `MarkandSweepGC` which automatically calls `clear()`
// when it goes out of scope. We do this because we want to make
// sure that all `*mut DummyObject` are freed from the heap.
impl Drop for MarkandSweepGC {
    fn drop(&mut self) {
        self.clear();
    }
}

impl MarkandSweepGC {
    fn new_spawn_test_dummies(amount: usize) -> Self {
        let mut roots = Vec::with_capacity(amount);
        for _ in 0..amount {
            let dummy_obj = DummyObject::new_on_heap();
            roots.push(dummy_obj);
        }

        Self { roots }
    }

    fn clear(&mut self) {
        for root in &mut self.roots {
            unsafe {
                // `ptr_head` is the head of the `root` path
                let mut ptr_head = *root;

                // Perform a deallocation loop until `ptr_head.next` is null,
                // which means that we have reached the end of the `root` path.
                while !(*ptr_head).next.is_null() {
                    // Get Address of next `DummyObject`
                    let next_addr = (*ptr_head).next;

                    // Deallocate the current `DummyObject`
                    DummyObject::free(ptr_head);

                    // `ptr_head` now is pointing/chasing the next node
                    // which was pointed to by the previous node.
                    ptr_head = next_addr;
                }

                // Free the current `ptr_head` which is the last node along
                // a given `root` path.
                // PERF: Is this branch needed here?
                if !ptr_head.is_null() {
                    DummyObject::free(ptr_head);
                }

                // Set `root` to null, disallowing further pointer chasing
                // of the current `root`.
                *root = std::ptr::null_mut();
            }
        }
    }

    fn refernce_dummy_at_to(&mut self, at: usize, dummy: *mut DummyObject) {
        let root = self.roots.get_mut(at);

        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        unsafe {
            (**root.unwrap()).reference_to(dummy);
        }
    }

    fn display_root_trail_addresses(&self, at: usize) {
        let root = self.roots.get(at);
        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        let root = root.unwrap();

        if root.is_null() {
            let msg =
                format!("[ALERT]: Root `DummyObject` was null at index {at}, so nothing to print!");
            println!("{msg}");
            return;
        }

        unsafe {
            print!("Root {at} path ({:#p}): ", *root);
            let mut ptr_head = (**root).next;
            while !(*ptr_head).next.is_null() {
                print!("{:#p} -> ", ptr_head);
                ptr_head = (*ptr_head).next;
            }

            if (*ptr_head).next.is_null() {
                print!("{:#p} -> ", ptr_head);
                println!("NULL");
            }
        }
    }

    fn display_root_trail_values(&self, at: usize) {
        let root = self.roots.get(at);
        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        let root = root.unwrap();

        if root.is_null() {
            let msg =
                format!("[ALERT]: Root `DummyObject` was null at index {at}, so nothing to print!");
            println!("{msg}");
            return;
        }

        unsafe {
            print!("Root {at} path ({:#?}): ", **root);
            let mut ptr_head = (**root).next;
            while !(*ptr_head).next.is_null() {
                print!("{:#?} -> ", &*ptr_head);
                ptr_head = (*ptr_head).next;
            }

            if (*ptr_head).next.is_null() {
                print!("{:#?} -> ", &*ptr_head);
                println!("NULL");
            }
        }
    }
}

fn main() {
    const DUMMY_AMOUNT: usize = 10;
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
        gc_arena.refernce_dummy_at_to(i, ptrs[i]);
    }

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
