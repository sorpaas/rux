use cap::Capability;
use common::*;

pub mod boxed;



pub fn retype_page_table(parent: &mut UntypedMemmoryCapability)

pub fn retype<F>(parent: &mut Capability, child: Capability, align: usize, f: F) -> Option<Box<Capability>>
    where F : FnOnce(start: PhysicalAddress, length: usize) -> {
    // What we get, as the parent, should be a untyped.
    parent.as_untyped_memory()
        .and_then(|parent_ptr| {
            
            if (parent_ptr.first_child.is_none()) {
                
            }
        })
        .or_else(|| {

        })

    assert!(parent.is_untyped_memory(), "Retype should be on a untyped memory capability.");

    // If parent doesn't have a child, we are done.
    if (parent.first_child.is_none()) {
        match(*parent) {
            
        }
    }

    // Find free space for allocation
    let mut last_cap = 
}

pub fn untype(cap: Capability) {
    
}

/// This function finds the next available free memory in an untyped capability.

fn find_next_free_address(parent: &UntypedMemoryCapability, size: usize, align: usize) -> usize {
    
}
