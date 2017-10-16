## Rux, a microkernel written in Rust

Rux is a hobbyist microkernel written in Rust, featuring a
capability-based system similar to [seL4](https://sel4.systems/).

- [Repository](https://source.that.world/source/rux/)
- [Documentation](https://that.world/~docs/rux/kernel/)

## Overview

Rux's goal is to become a safe general-purpose microkernel. It tries to
take advantage of Rust's memory model -- ownership and lifetime. While
the kernel will be small, unsafe code should be kept minimal. This makes
updating functionalities of the kernel hassle-free.

Rux uses a design that is similar to seL4. While there won't be formal
verification in the short term, it tries to address some design issues
of seL4, for example, capability allocation.

## Contributing

We use [Phabricator](https://source.that.world/source/rux/) to manage
code reviews and collabration. To submit a patch, install
[arcanist](https://secure.phabricator.com/book/phabricator/article/arcanist_quick_start/). After
you finished coding, run `arc diff` and it will guide you to create a
new patch in Phabricator.

If you don't like arcanist, you can also submit raw diff directly
through the [web
interface](https://source.that.world/differential/diff/create/). Refer
to guide in [this page](https://llvm.org/docs/Phabricator.html) for
details on how to do this.

## Quickstart

Currently, due to packaging problem, the kernel is only tested to
compile and run on Linux with `x86_64`. Platforms with qemu and compiler
target of `x86_64` should all be able to run this kernel, but do it at
your own risk.

To run the kernel, first install `Rust`, `qemu`, and cross-compiled
GNU's `binutils`. The easiest way to do it is through the
`default.nix` file provided in the source code. Install
[Nix](http://nixos.org/nix/), then go to the source code root and run
the following command:

```lang=bash
nix-shell
```

After that, run:

```lang=bash
make run
```

You should see the kernel start to run with a qemu VGA buffer. The
buffer, after the kernel successfully booted, should show a simple
command-line interface controlled by `rinit` program launched by the
kernel. Several commands can be used to test things out.

```lang=bash
echo [message]
```

Echo messages and print them back to the VGA buffer.

```lang=bash
list
```

Print the current `CPool` slots into the kernel message buffer.

```lang=bash
retype cpool [source slot id] [target slot id]
```

Retype an Untyped capability into a CPool capability. `[source slot
id]` should be a valid slot index of an Untyped capability. `[target
slot id]` should be an empty slot for holding the retyped CPool
capability.

### Example: Talk With a Child Task

The rinit program will start the command line interface when it is the
first to run. For all subsequent rinit programs, they will wait on a
channel (using the root CPool of index 255), and print out the value
to the serial buffer.

When you see the command line in the qemu VGA buffer, the "parent"
rinit program has been successfully started. We can then create a
"child" program using the same memory layout (sharing one page table),
and make them talk.

When the "parent" rinit program is started, you should see something
like below in the VGA buffer:

```
Child entry should be at: 0x88b0 (34992)
Child stack pointer should be at: 0x70003ffc (1879064572)
```

Those messages are useful if we want to create a "child".

To do this, we first retype a new task from an untyped capability.

```lang=bash
retype task 2 249
```

This creates a new task in "inactive" state, which allows us to do
further settings. We then set its stack pointer and instruction
pointer to the valid value:

```lang=bash
set stack 249 1879064572
set instruction 249 34992
```

Then we set the task's root CPool and top page table the same as the
"parent":

```lang=bash
set cpool 249 0
set table 249 3
```

The task buffer is used for system calls, thus we need a new one for
the child. Fortunately, in the kernel `kmain`, we have already created
one at index 250, so we can set that as the "child"'s buffer.

```lang=bash
set buffer 249 250
```

After that, we can set the state of the task to active. This will
start the task.

```lang=bash
set active 249 1
```

If you are lazy and don't want to create the task from scratch. The
command below automates the task from retyping tasks from untyped to
activating the task.

```lang=bash
start child
```

After the child has started, we can send numbers to channel (with
CPool index 255).

```lang=bash
send raw 5
```

You should see `[kernel] Userspace print: Received from master: 5` in
the serial message buffer.

You can also send capabilities over the channel. Rux uses different
system calls to send raw values, payloads and capabilities through
channels. If you wish to send capabilities, modify `let value: u64 =
system::channel_take(CAddr::from(255));` in `child_main` of
`rinit/src/lib.rs` to `let value: CAddr =
system::channel_take_cap(CAddr::from(255));`.

```lang=bash
send cap 0
```

And the top-level capability pool capability (CPoolCap) is copied
again from parent to child.

## Source Code Structure

The development of Rux happen in the `master` branch in the source code
tree. The kernel resides in the `kernel` folder, with platform-specific
code in `kernel/src/arch`. For the `x86_64` platform, the kernel is
booted from `kernel/src/arch/x86_64/start.S`. The assembly code them
jumps to the `kinit` function in `kernel/src/arch/x86_64/init/mod.rs`.

After the kernel is bootstrapped, it will initialize a user-space
program called `rinit`, which resides in the `rinit` folder. The
user-space program talks with the kernel through system calls, with ABI
defined in the package `abi`, and wrapped in `system`.

## Kernel Design

### Capabilities

Capabilities are used in kernel to manage Kernel Objects. Those
Capabilities are reference-counted pointers that provide management for
object lifecycles.

Capabilities in user-space can be accessed using so-called `CAddress`,
refered through the root capability of the user-space task. This helps
to handle all permission managements for the kernel, and thus no
priviliged program or account is needed.

Current implemented capabilities are:

- Untyped memory capability (UntypedCap)
- Capability pool capability (CPoolCap)
- Paging capability
  - PML4Cap, PDPTCap, PDCap, PTCap
  - RawPageCap, TaskBufferPageCap
  - VGA buffer
- CPU time sharing capability (TaskCap)
- Inter-process communication capability (ChannelCap)

#### Example: Initialize a New Task

This example shows how to initialize a new task using the capability
system.

- Create an empty TaskCap.
- Create an empty CPoolCap.
- Initialize paging capabilities (One PML4Cap, Several PDPTCap, PDCap,
  PTCap and RawPageCap)
- Assign the stack pointer in TaskCap.
- Load the program into those RawPageCap.
- Assign the PML4Cap to TaskCap.
- Assign the CPoolCap to TaskCap.
- Switch to the task!

#### Implementation

Implementing reference-counted object is a little bit tricky in kernel,
as objects need to be immediately freed, and all weak pointers need to
be cleared after the last strong pointer goes out. Rux's implementation
uses something called `WeakPool` to implement this. The original
reference counted object (called `Inner`), form a double-linked list
into the nodes in multiple WeakPools.

### Capability Pools

Capability Pools (or `CPool`) are used to hold multiple capability
together. This is useful for programs to pass around permissions, and is
essential for `CPool` addressing. In implementation, capability pools
are implemented as a `WeakPool`.

### Tasks

A task capability has a pointer to a capability pool (the root for
`CPool` addressing), a task buffer (for kernel calls), and a top-level
page table. When switching to a task, the kernel switches to the page
table specified.

The `switch_to` function implemented uses several tricks to make it
"safe" as in Rust's sense. When an interrupt happens in userspace, the
kernel makes it as if the `switch_to` function has returned.

In kernel-space, interrupts are disabled.

### Channels

Tasks communicate with each other through channels. A channel has a
short buffer holding messages sent from a task, and will respond this to
the first task that calls `wait` on the channel.
