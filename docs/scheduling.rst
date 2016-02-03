Scheduling
==========

Rux delegates scheduling to the user mode. Timer interrupt is treated as a
normal IPC, which sends a message to a schedule -- a special thread that holds a
**scheduling capability**. The scheduler has all the control over processor time
until other kernel events happen. A scheduler, as well as other threads, can
delegate its time to other threads, which makes scheduling possible. At the same
time, a blocked thread can also delegate its time to the scheduler, which makes
scheduling more efficient. Delegation is done by an IPC to the kernel.

Because of the user mode scheduling, we now need two kernel calls to finish a
preemption, which could potentially make our kernel run slow.
