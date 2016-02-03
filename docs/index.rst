.. Rux documentation master file, created by
   sphinx-quickstart on Thu Jan 28 23:05:04 2016.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Welcome to Rux's documentation!
===============================

Rux is a L4-family microkernel implemented in Rust. As a microkernel, Rux tries
to delegate as much work as possible to the user mode. Several things are
implemented (or plan to be implemented) in Rux:

* Resource isolation
* Memory management
* Task scheduling
* Interrupts
* Inter-process communication
* Hardware primitives

Contents:

.. toctree::
   :maxdepth: 2

   capabilities
   scheduling
   resources

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`

