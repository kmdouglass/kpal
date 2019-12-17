# KPAL Plugins

[![Docs.rs](https://docs.rs/kpal-plugin/badge.svg)](https://docs.rs/kpal-plugin)

A library that lets you incorporate new peripherals into
[KPAL](https://github.com/kmdouglass/kpal).

## Overview

`kpal-plugin` provides the data types and functions that allow you to write your own plugin for
KPAL. A plugin provides the common interface between the KPAL daemon and a specific peripheral. You
expose functions through the library that are part of the plugin API, and these in functions call
your own custom code that work with your peripheral.

Plugins are implemented as shared libraries (`.so` files on Linux).

## Getting started

Check out the [examples](examples) for ideas on how to write a plugin.
