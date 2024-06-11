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
        let ptr = unsafe { std::alloc::alloc(layout) };

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

            unsafe {
                while !(*ptr).next.is_null() {
                    ptr = (*ptr).next;
                }
            }

            unsafe {
                (*ptr).next = dummy;
            }
        }
    }
}

/// This is a memory arena that keeps track of the references to roots of `DummyObject`
#[derive(Debug, Default)]
struct MarkandSweepGC {
    roots: Vec<*mut DummyObject>,
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
            let msg = "[ALERT]: Root `DummyObject` was null at index {at}, so nothing to print!";
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
            let msg = "[ALERT]: Root `DummyObject` was null at index {at}, so nothing to print!";
            println!("{msg}");
            return;
        }

        unsafe {
            print!("Root {at} path ({:#?}): ", *root);
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
    let mut gc_arena = MarkandSweepGC::new_spawn_test_dummies(5);

    let ptr = DummyObject::new_on_heap();

    // unsafe {
    //     dbg!(&*(ptr));
    // }

    {
        let d3 = DummyObject::new_on_heap();
        let d4 = DummyObject::new_on_heap();
        let d5 = DummyObject::new_on_heap();

        let root_idx = 0;
        println!("[Before adding reference at index {root_idx}]");
        println!("{gc_arena:#?}");
        gc_arena.refernce_dummy_at_to(root_idx, d3);
        gc_arena.refernce_dummy_at_to(root_idx, d4);
        gc_arena.refernce_dummy_at_to(root_idx, d5);
        println!("\n\n\n[After adding reference at index {root_idx}]");
        println!("{gc_arena:#?}");

        gc_arena.display_root_trail_addresses(root_idx);
    }
    gc_arena.display_root_trail_values(0);
}
