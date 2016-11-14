pub type APICSlot = [u8; 16];

#[derive(Debug, Clone)]
#[repr(C)]
pub struct APICPage {
    reserved1: [APICSlot; 2],
    pub local_apic_id: APICSlot,
    pub local_apic_version: APICSlot,
    reserved2: [APICSlot; 4],
    pub task_priority: APICSlot,
    pub arbitration_priority: APICSlot,
    pub processor_priority: APICSlot,
    pub eoi: APICSlot,
    pub remote_read: APICSlot,
    pub local_destionation: APICSlot,
    pub destination_format: APICSlot,
    pub spurious_interrupt_vector: APICSlot,
    pub in_service: [APICSlot; 8],
    pub trigger_mode: [APICSlot; 8],
    pub interrupt_request: [APICSlot; 8],
    pub error_status: APICSlot,
    reserved3: [APICSlot; 6],
    pub lvt_cmci: APICSlot,
    pub interrupt_command: [APICSlot; 2],
    pub lvt_timer: APICSlot,
    pub lvt_thermal_sensor: APICSlot,
    pub lvt_performance_monitoring_counters: APICSlot,
    pub lvt_lint0: APICSlot,
    pub lvt_lint1: APICSlot,
    pub lvt_error: APICSlot,
    pub initial_count: APICSlot,
    pub current_count: APICSlot,
    reserved4: [APICSlot; 4],
    pub divide_configuration: APICSlot,
    reserved5: [APICSlot; 1]
}
