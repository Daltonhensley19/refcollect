use std::cell::Cell;

use rand::{thread_rng, Rng};

#[derive(Debug, Clone)]
pub struct DummyObject {
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

    pub fn new_on_heap() -> *mut DummyObject {
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

    fn free(object: &*mut DummyObject) {
        assert!(
            !object.is_null(),
            "Memory deallocation failed due to trying to free a null pointer!"
        );
        let layout = std::alloc::Layout::new::<DummyObject>();

        // SAFETY: `object` must refer to a chuck of memory that is live
        // and allocated/aligned to a `layout` of `DummyObject`.
        println!("Currently freeing: {:#?}", &*object);
        unsafe { std::alloc::dealloc(*object as *mut u8, layout) };
    }
}

/// This is a memory arena that keeps track of the references to roots of `DummyObject`
#[derive(Debug, Default)]
pub struct MarkandSweepGC {
    roots: Vec<Cell<*mut DummyObject>>,
    leak: bool,
}

// RAII for `MarkandSweepGC` which automatically calls `clear()`
// when it goes out of scope. We do this because we want to make
// sure that all `*mut DummyObject` are freed from the heap.
impl Drop for MarkandSweepGC {
    fn drop(&mut self) {
        if !self.leak {
            self.clear();
        }
    }
}

impl MarkandSweepGC {
    pub fn new_with_test_dummy_roots(amount: usize) -> Self {
        let mut roots = Vec::with_capacity(amount);
        for _ in 0..amount {
            let dummy_obj = Cell::new(DummyObject::new_on_heap());
            roots.push(dummy_obj);
        }

        Self { roots, leak: false }
    }

    pub fn leak(&mut self) {
        self.leak = true;
    }

    pub fn display_root(&self, root_idx: usize) {
        let root = self.roots.get(root_idx);

        if root.is_none() {
            return;
        }

        let root = root.unwrap();

        unsafe {
            if (root.get()).is_null() {
                return;
            }

            println!("{:#?}", *root.get());
        }
    }

    pub fn display_root_address(&self, root_idx: usize) {
        let root = self.roots.get(root_idx);

        if root.is_none() {
            return;
        }

        let root = root.unwrap();

        if (root.get()).is_null() {
            return;
        }

        println!("{:#?}", root.get());
    }

    pub fn display_roots(&self) {
        for idx in 0..self.roots.len() {
            self.display_root(idx);
        }
    }

    pub fn display_roots_addresses(&self) {
        for idx in 0..self.roots.len() {
            self.display_root_address(idx)
        }
    }

    pub fn add_root_with(&mut self, root: *mut DummyObject) {
        self.roots.push(Cell::new(root));
    }

    pub fn add_root(&mut self) {
        let root = DummyObject::new_on_heap();
        self.roots.push(Cell::new(root));
    }

    pub fn add_n_roots(&mut self, amount: usize) {
        for _ in 0..amount {
            self.add_root();
        }
    }

    fn clear(&mut self) {
        for root in &mut self.roots {
            unsafe {
                // `ptr_head` is the head of the `root` path
                let mut ptr_head = root.get();

                // If the current `root` is null, then we have
                // have no need to deallocate anything since it was likely
                // deallocated already.
                if ptr_head.is_null() {
                    continue;
                }

                // Perform a deallocation loop until `ptr_head.next` is null,
                // which means that we have reached the end of the `root` path.
                while !(*ptr_head).next.is_null() {
                    // Get Address of next `DummyObject`
                    let next_addr = (*ptr_head).next;

                    // Deallocate the current `DummyObject`
                    DummyObject::free(&ptr_head);

                    // `ptr_head` now is pointing/chasing the next node
                    // which was pointed to by the previous node.
                    ptr_head = next_addr;
                }

                // Free the current `ptr_head` which is the last node along
                // a given `root` path.
                // PERF: Is this branch needed here?
                if !ptr_head.is_null() {
                    DummyObject::free(&ptr_head);
                }

                // Set `root` to null, disallowing further pointer chasing
                // of the current `root`.
                root.set(std::ptr::null_mut());
            }
        }
    }

    pub fn refernce_dummy_at_to(&mut self, at: usize, dummy: *mut DummyObject) {
        let root = self.roots.get_mut(at);

        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        unsafe {
            (*root.unwrap().get()).reference_to(dummy);
        }
    }

    pub fn display_root_trail_addresses(&self, at: usize) {
        let root = self.roots.get(at);
        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        let root = root.unwrap();

        if root.get().is_null() {
            let msg =
                format!("[ALERT]: Root `DummyObject` was null at index {at}, so nothing to print!");
            println!("{msg}");
            return;
        }

        unsafe {
            print!("Root {at} path ({:#p}): ", root.get());
            let mut ptr_head = (*root.get()).next;
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

    pub fn display_root_trail_values(&self, at: usize) {
        let root = self.roots.get(at);
        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        let root = root.unwrap().get();

        // dbg!((*root).is_null());
        if root.is_null() {
            let msg =
                format!("[ALERT]: Root `DummyObject` was null at index {at}, so nothing to print!");
            println!("{msg}");
            return;
        }

        unsafe {
            // assert!((*root).is_null());
            print!("Root {at} path ({:#?}): ", *root);
            let mut ptr_head = (*root).next;
            while !ptr_head.is_null() && !(*ptr_head).next.is_null() {
                print!("{:#?} -> ", &*ptr_head);
                ptr_head = (*ptr_head).next;
            }
            if ptr_head.is_null() {
                println!(" -> NULL");
                assert!(false);
            }
            if (*ptr_head).next.is_null() {
                print!("{:#?} -> ", &*ptr_head);
                println!("NULL");
            }
        }
    }

    pub fn mark_unreachable(&mut self, root_idx: usize, mut root_depth: usize) {
        let root = self.roots.get(root_idx);
        assert!(
            root.is_some(),
            "No root dummy object was not found at index"
        );

        let root = root.unwrap().get();

        if root.is_null() {
            let msg = format!(
                "[ALERT]: Root `DummyObject` was null at index {root_idx}, so nothing to mark!"
            );
            println!("{msg}");
            return;
        }

        let mut current_depth = 0usize;
        let target_depth = root_depth;
        unsafe {
            let mut ptr_head = root;
            while root_depth != 0 {
                if (*ptr_head).next.is_null() {
                    break;
                }

                ptr_head = (*ptr_head).next;
                root_depth = root_depth.saturating_sub(1);
                current_depth = current_depth.saturating_add(1);
            }

            // Mark current `ptr_head` as marked, meaning that it
            // is no longer reachable.
            (*ptr_head).marked = true;
        }

        if current_depth != target_depth {
            assert!(false, "No dummy object to mark was found on path to root");
        }
    }

    pub fn sweep(&mut self) {
        println!("[INFO] Sweeping garbage...");
        for (idx, root) in self.roots.iter().enumerate() {
            let ptr_head = root.get();

            if ptr_head.is_null() {
                continue;
            }

            // Move `ptr_head` into a `Cell`, as it allows us to mutate/free
            // marked `DummyObject`s via interior mutability.
            let mut ptr_head = ptr_head;

            unsafe {
                // If the `root` itself is marked, then we start
                // the sweep beginning at the `root`.
                let mut last_unmarked_addr = root.get();
                if (*ptr_head).marked {
                    self.sweep_path_starting_at(ptr_head);

                    self.roots[idx].set(std::ptr::null_mut());

                    continue;
                }

                while !(*ptr_head).next.is_null() {
                    // If the next address is marked, save the current
                    // address as we will have to "disconnect" it from
                    // the rest of the path.
                    if (*(*ptr_head).next).marked {
                        last_unmarked_addr = ptr_head;
                    }

                    ptr_head = (*ptr_head).next;

                    if (*ptr_head).marked {
                        self.sweep_path_starting_at(ptr_head);

                        // Disconnect the last accessable DummyObject from the sweeped
                        // part of the ahead of it.
                        (*last_unmarked_addr).next = std::ptr::null_mut();
                    }
                }
            }
        }

        println!("[INFO] Sweeping garbage...DONE\n\n");
    }

    // Allows the caller to simulate the marking of a `DummyObject`
    //
    // Parameters:
    // root_idx: Index of the root `DummyObject`
    // self: &mut MarkandSweepGC
    fn mark_root(&mut self, root_idx: usize) {
        let root = self.roots.get(root_idx);

        if root.is_none() {
            return;
        }

        let root = root.unwrap().get();

        unsafe {
            if root.is_null() {
                return;
            }

            println!("[INFO] Marking root {root:#?}...");
            (*root).marked = true;
            println!("[INFO] Marking root {root:#?}...DONE\n\n");
        }
    }

    unsafe fn sweep_path_starting_at(&self, mut ptr_head: *mut DummyObject) {
        // While we are not at the end of the `ptr_head` path
        let mut next_addr;
        while !(*ptr_head).next.is_null() {
            // Get next address since it is destroyed when we
            // free the current `ptr_head`.
            next_addr = (*ptr_head).next;

            DummyObject::free(&ptr_head);

            // Move `ptr_head` to point to next `DummyObject`
            ptr_head = next_addr;
        }

        // Free the remaining `ptr_head` at the end of the `root` path.
        // TODO: Make sure this does not segfault
        DummyObject::free(&ptr_head);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
