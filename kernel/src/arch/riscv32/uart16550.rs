const UART_ADDRESS: usize = 0x10000000;

const UART_QUEUE_ADDRESS: usize = UART_ADDRESS + 0;
const UART_LINESTAT_ADDRESS: usize = UART_ADDRESS + 5;

const UART_STATUS_RX: u8 = 0x01;
const UART_STATUS_TX: u8 = 0x20;

pub unsafe fn getchar() -> Option<u8> {
    let uart_linestat: *mut u8 = UART_LINESTAT_ADDRESS as *mut u8;
    let uart_queue: *mut u8 = UART_QUEUE_ADDRESS as *mut u8;

    if *uart_linestat & UART_STATUS_RX != 0 {
        Some(*uart_queue)
    } else {
        None
    }
}

pub unsafe fn putchar(ch: u8) {
    let uart_linestat: *mut u8 = UART_LINESTAT_ADDRESS as *mut u8;
    let uart_queue: *mut u8 = UART_QUEUE_ADDRESS as *mut u8;

    // while *uart_linestat & UART_STATUS_TX == 0 { }
    *uart_queue = ch;
}
