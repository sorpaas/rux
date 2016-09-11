use arch::interrupt::{IDT};

pub fn init() {
    IDT.load();
}
