# Wyzoid 🧶

[![Crates.io](https://img.shields.io/crates/v/wyzoid)](https://crates.io/crates/wyzoid)
[![Build Status](https://travis-ci.org/maeln/wyzoid.svg?branch=master)](https://travis-ci.org/maeln/wyzoid)

Wyzoid is a small framework made to easily experiment with compute shader / GPGPU using Vulkan (via [ash](https://crates.io/crates/ash)).

## Examples

The project include 3 examples:

1. "basic": Execute one shader on one buffer
   - `cargo run --example basic`
2. "multiplebuffer": Execute one shader on two buffer
   - `cargo run --example multiplebuffer`
3. "multiplebuffershader": Execute two shader in series on two buffer
   - `cargo run --example multiplebuffershader`

## Documentation

Documentation is very much a todo. In the meanwhile, you can look at the [examples](./examples).
