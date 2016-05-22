use common::*;

pub fn necessary_page_count(object_size: usize) -> usize {
    if object_size % PAGE_SIZE == 0 {
        object_size / PAGE_SIZE
    } else {
        object_size / PAGE_SIZE + 1
    }
}
