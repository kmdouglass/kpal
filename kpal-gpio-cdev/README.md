# kpal-gpio-cdev

[![Docs.rs](https://docs.rs/kpal-gpio-cdev/badge.svg)](https://docs.rs/kpal-gpio-cdev)

A KPAL plugin for the Linux GPIO character device API.

## Overview

`kpal-gpio-cdev` is a wrapper around the Rust
[gpio-cdev](https://github.com/rust-embedded/gpio-cdev) library. The [GPIO character device
ABI](https://www.kernel.org/doc/Documentation/ABI/testing/gpio-cdev) is the modern interface
between users and a Linux kernel that is controllig GPIO devices.

## Getting started

### Cross-compiling for ARMv7

The ARMv7 instruction set architecture (ISA) is used by the Raspberry Pi Model 3, among other
systems. To compile this plugin for the ARMv7 first make sure that you have an ARMv7 Rust toolchain
installed. You will also need a linker. On Ubuntu-like systems, this can be obtained from one of
many packages, such as
[gcc-arm-linux-gnueabihf](https://packages.ubuntu.com/bionic/devel/gcc-arm-linux-gnueabihf):

```console
$ rustup target add armv7-unknown-linux-gnueabihf
$ sudo apt install gcc-arm-linux-gnueabihf
```

To configure the linker for this architecture, add the following lines to `$HOME/.cargo/config`:

```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

To build the library for the ARMv7 ISA, run the following from this directory:

```console
cargo build --target=armv7-unknown-linux-gnueabihf
```

The artifacts should be found inside the `target` directory of the `kpal` root directory
`../target/armv7-unknown-linux-gnueabihf`.
