# Kyle's Peripheral Abstraction Layer 

[![CircleCI](https://circleci.com/gh/kmdouglass/kpal.svg?style=svg)](https://circleci.com/gh/kmdouglass/kpal)
[![Docs.rs](https://docs.rs/kpal/badge.svg)](https://docs.rs/kpal)
[![Crates.io](https://img.shields.io/crates/v/kpal)](https://crates.io/crates/kpal)

KPAL is an extensible control system for physical computing.

## Documentation

**KPAL is under development. The API will not be considered stable until 1.0 is released.**

- [kpal-plugin](https://docs.rs/kpal-plugin/0.1.0/kpal_plugin/) - Used to write plugins for KPAL
- [kpal-gpio-cdev](https://docs.rs/kpal-gpio-cdev/0.1.0/kpal_gpio_cdev/) - Controls the GPIO pins
  on a Raspberry Pi

## Overview

KPAL allows you to control and read data from peripherals attached to a computer such as your
desktop or Raspberry Pi. It acts as an interface between users and individual peripherals through
two application programming interfaces (APIs):

- **the user API** A web service that can be accessed from different computers on a network,
  including smart phones
- **the plugin API** A high-level plugin interface that allows KPAL to communicate with
  peripherals such as senors, motors, and cameras

## Quickstart

1. Download the archive that matches the latest version of the binaries for your platform from the
   [releases page](https://github.com/kmdouglass/kpal/releases).
2. Unpack the archive.
3. Create the following folder in your home directory:

```console
mkdir -p ~/.kpal/libraries
```
4. Move the file `libbasic-plugin.so` from the archive into the `~/.kpal/libraries` folder. This is
   an example plugin that is used for demonstrations and testing; it does not control any actual
   hardware.
5. Run the binary file `kpald` to start the daemon. If you want to see the logs, set the `RUST_LOG`
   environment variable to `info`, `error`, or `debug`, depending on the desired log level:

```console
RUST_LOG=info ./kpald
```

You may now make HTTP requests to the daemon. The following examples use the UNIX `curl` command
line utility to make the requests, but you may use the HTTP client of your choice.

```console
# Get the libraries that are available to the daemon
curl -s localhost:8000/api/v0/libraries

# Get the library with ID 0
curl -s localhost:8000/api/v0/libraries/0

# Create a new peripheral from the library with ID 0
curl -s \
     --request POST \
     localhost:8000/api/v0/peripherals \
     --header "Content-Type: application/json" \
     --data '{"name":"foo","library_id":0}'

# Create a new peripheral and override the default value of a pre-init attribute
curl -s \
     --request POST \
     localhost:8000/api/v0/peripherals \
     --header "Content-Type: application/json" \
     --data '{
         "name": "foo",
         "library_id": 0,
         "attributes": [
             {"id":0, "variant":"double", "value": 999.99}
         ]
     }'

# Get all the peripherals currently managed by the daemon
curl -s localhost:8000/api/v0/peripherals

# Get the peripheral with ID 0
curl -s localhost:8000/api/v0/peripherals/0

# Get the attributes of the peripheral with ID 0
curl -s localhost:8000/api/v0/peripherals/0/attributes

# Get the attribute with ID 0 from the peripheral with ID 0
curl -s localhost:8000/api/v0/peripherals/0/attributes/0

# Set the value of the attribute with ID 0 of the peripheral with ID 0
curl -s \
     --request PATCH \
     localhost:8000/api/v0/peripherals/0/attributes/0 \
     --header "Content-Type: application/json" \
     --data '{"variant":"double","value":42}'
```

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
