Capabilities
============

Capabilities manages rights to resources in Rux. After bootstrap, the kernel
creates several *untyped capabilities* representing the free memory, as well as
*kernel reserved memory capabilities* for the kernel memory and boot information
memory blocks. Capabilities can be stored in *capability pools*, those
capability pool are either owned by kernel or by a particular thread. If a
thread owns a capability pool, it can use capabilities inside to call the kernel
to create new virtual memory, manage free memory, create inter-process calls,
pause other threads, and etc.

Untyped Memory Capability
-------------------------

**Untyped memory capability** represents free memory. It can do nothing useful
by itself, but can be *retyped* into more useful things.

* **Retype to page managed capabilities**: An untyped memory capability can be
  retyped into a memory that represents an object. Because memory needs to be
  mapped as virtual memory in order for kernel to access, those objects occupies
  the same size as one or more pages. Those page managed capabilities are
  created by temporarily mapping the required pages so as to create the memory,
  and then unmap them.
* **Retype to smaller untyped memory capability**: This is useful if a memory
  manager decides to allocate memory for other threads.

Page Managed Capability
-----------------------

**Page managed capability** allows mapping the underlying memory block and
represent it as a normal object in Rust. Under the hood, once mapped, the
capability will be bound to the page table capability. The bound is created as a
*reference-counted pointer*. As a result, once a capability is mapped, its page
table cannot be freed until the capability is unmapped. If the page table
capability loses its capability pool reference during that period, it will be
recaptured by the kernel and be delivered to a specific capability pool based on
kernel configuration.

Capability Pool Capability
``````````````````````````

**Capability pool capability** is a type of page managed capability. Its
underlying object stores a fixed number of other capabilities. A capability pool
capability is either owned by kernel or by a thread. In the second case, the
thread also owns all the capabilities inside the pool.
