# Constelation kernel libraries
> A set of libraries used by the Constellation OS kernel


# Overview

Constelation is my hobby project — an operating system written in the Rust programming language. This repository contains all the libraries used by the Constelation kernel.

The main goal of this project is to provide a modern operating system that is secure by design.

# Libraries

## `libmem`
The `libmem` library provides basic functions for memory management. These functions include several simple abstractions, an extent allocator, and a global allocator.

## `libpages`
`libpages` is built on top of the `libmem` library and provides a subsystem for managing virtual address spaces. This includes a physical frame allocator based on the extent allocator from `libmem`, a grass-fed paging implementation, quickly mapped pages to speed up memory management, and several other systems

## `liblock`
This library provides synchronization and initialization primitives such as `Mutex`, `RwLock`, and `LazyLock`

## `libelf`
`libelf` is responsible for parsing and loading ELF files. For security reasons, it focuses primarily on PIE (**P**osition-**I**ndependent **E**xecutable) files so that `libpages` can randomize their virtual address space

## `libarch`
`libarch` does not yet exist, but it will be a very important library responsible for CPU management. It will provide functionality that depends on the current processor architecture. This includes, for example, management of floating-point and SIMD registers, interrupts, context switching, and much more

## `libtest`
`libtest` serves as a bridge between the tested libraries, which are `#![no_std]` and `std`—that is, the test runtime. It contains a simple RNG for fuzzing and macros for printing text.
