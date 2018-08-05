use core::fmt;

pub struct Writer;

impl Writer {
	/// Obtain a logger for the specified module
	pub fn get(module: &str) -> Writer {
		let mut ret = Writer;

		// Print the module name before returning (prefixes all messages)
		{
			use core::fmt::Write;
			let _ = write!(&mut ret, "[{}] ", module);
		}

		ret
	}
}

impl ::core::ops::Drop for Writer {
	fn drop(&mut self)
	{
		// Write a terminating newline before releasing the lock
		{
			use core::fmt::Write;
			let _ = write!(self, "\n");
		}
	}
}

impl fmt::Write for Writer {
	fn write_str(&mut self, s: &str) -> fmt::Result
	{
		unsafe {
            for b in s.bytes() {
				::arch::putchar(b);
            }
		}
		Ok(())
	}
}
