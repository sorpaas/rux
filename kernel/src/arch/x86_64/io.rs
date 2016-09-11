#![allow(dead_code)]	// < This sample doesn't use them, but you might :)

// NOTE: This code uses "{dx}N" as a register specifier. I _believe_ the N means (8-bit immediate)

/// Write a byte to the specified port
pub unsafe fn outb(port: u16, val: u8)
{
	asm!("outb $0, $1" : : "{al}"(val), "{dx}N"(port));
}

/// Read a single byte from the specified port
pub unsafe fn inb(port: u16) -> u8
{
	let ret : u8;
	asm!("inb $1, $0" : "={al}"(ret) : "{dx}N"(port));
	return ret;
}

/// Write a word (16-bits) to the specified port
pub unsafe fn outw(port: u16, val: u16)
{
	asm!("outw $0, $1" : : "{ax}"(val), "{dx}N"(port));
}

/// Read a word (16-bits) from the specified port
pub unsafe fn inw(port: u16) -> u16
{
	let ret : u16;
	asm!("inw $1, $0" : "={ax}"(ret) : "{dx}N"(port));
	return ret;
}

/// Write a long/double-word (32-bits) to the specified port
pub unsafe fn outl(port: u16, val: u32)
{
	asm!("outl $0, $1" : : "{eax}"(val), "{dx}N"(port));
}

/// Read a long/double-word (32-bits) from the specified port
pub unsafe fn inl(port: u16) -> u32
{
	let ret : u32;
	asm!("inl $1, $0" : "={eax}"(ret) : "{dx}N"(port));
	return ret;
}
