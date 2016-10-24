/// A very primitive logging macro
///
/// Obtaines a logger instance (locking the log channel) with the current module name passed
/// then passes the standard format! arguments to it
macro_rules! log{
	( $($arg:tt)* ) => ({
		// Import the Writer trait (required by write!)
		use core::fmt::Write;
		let _ = write!(&mut ::logging::Writer::get(module_path!()), $($arg)*);
	})
}

macro_rules! normal_half {
    ( $t:ty ) => {
        impl !Send for $t { }
        impl !Sync for $t { }

        impl CapHalf for $t {
            fn mark_deleted(&mut self) {
                self.deleted = true;
            }
        }

        impl Drop for $t {
            fn drop(&mut self) {
                assert!(self.deleted, "attempt to drop unmarked half {:?}", self);
            }
        }
    }
}
