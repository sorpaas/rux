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

macro_rules! global_variable {
    ( $get:ident, $get_mut:ident, $var:ident, $t:ty, $init_v:expr) => {
        static mut $var: Option<Unique<$t>> = $init_v;
        
        pub fn $get() -> &'static $t {
            unsafe {
                match $var {
                    Some(ref x) => x.get(),
                    None => panic!()
                }
            }
        }

        pub fn $get_mut() -> &'static mut $t {
            unsafe {
                match $var {
                    Some(ref mut x) => x.get_mut(),
                    None => panic!()
                }
            }
        }
    }
}

macro_rules! global_const {
    ( $get:ident, $var:ident, $t:ty, $init_v:expr) => {
        static mut $var: Option<$t> = $init_v;

        pub fn $get() -> $t {
            unsafe {
                match $var {
                    Some(x) => x,
                    None => panic!()
                }
            }
        }
    }
}
