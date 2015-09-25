# cargo rumpbake

A small wrapper around `cargo build` and `rumpbake` for generating rumprun
unikernel images from binary crates.

## Installation

The prefered way to use this build and install this subcommand is by
building the `rust` package from
[rumprun-packages](https://github.com/rumpkernel/rumprun-packages).

## Use

Make sure rumprun's `app-tools` is in your `$PATH`. A crate named *"hello"*
with a single binary target can be baked into a rumprun unikernel as follows.

    cargo rumpbake hw_virtio

Which is roughtly equivalent to the following:

    cargo build --target x86_64-rumprun-netbsd
    rumpbake hw_virtio hello.img ./target/x86_64-rumprun-netbsd/debug/hello

The name of the generated image can be set using the `--output` flag. Use the
`rumprun` command line utility to execute the generated image:

    rumprun qemu -i hello.img

Refer to the [rumpkernel wiki](http://wiki.rumpkernel.org/Repo:-rumprun) for
more information.
