# Kyle's Peripheral Abstraction Layer [![CircleCI](https://circleci.com/gh/kmdouglass/kpal.svg?style=svg)](https://circleci.com/gh/kmdouglass/kpal)

KPAL is an extensible control system for physical computing.

**KPAL is under development. The API will not be considered stable until 1.0.**

## Overview

KPAL allows you to control and read data from peripherals attached to a computer such as your
desktop or Raspberry Pi. It acts as an interface between users and individual peripherals through
two application programming interfaces (APIs):

- **the user API** A web service that can be accessed from different computers on a network,
  including smart phones
- **the plugin API** A high-level plugin interface that allows KPAL to communicate with
  peripherals such as senors, motors, and cameras
  
## Core components

![High level architecture of KPAL](./resources/img/high_level_architecture.svg)

### Object model

The object model is the set of resources with which users interact. Currently, these resources
include:

- **peripherals** Models of individual hardware peripherals
  - **attributes** Values that represent the state of a peripheral
- **libraries** The shared libraries that enable the plugin API

### Daemon

The KPAL daemon, or `kpald`, is a web server that runs on the computer to which the peripherals are
connected. Users directly interact with the daemon through the user API. Each peripheral runs
inside its own thread which is spawned by a POST request to the user API. The daemon forwards other
user requests to each thread through the thread's dedicated channel. The threads interpret the
incoming requests and, in response, read and write data to individual plugins through the plugin
API using shared libraries.

### Plugins

Plugins are the means by which peripherals are integrated into KPAL. A plugin uses a shared library
(a `.so` file on Linux) to communicate with the daemon. The common set of functions that the
library provides is the plugin API. Any programming language that can provide a C language
interface can be used to write a plugin library.

A plugin combines the data that represents a peripheral's state with the functionality for
controlling the hardware device that is modeled by the peripheral.

## Writing plugins

See the [kpal-plugin](kpal-plugin) crate for more information on how to write your own plugins.

[kpal-gpio-cdev](kpal-gpio-cdev) is a plugin crate that controls the output of a single GPIO pin.
