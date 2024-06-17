# talktopirs
Talking to my raspberrypi(es) in Rust

# Prereqs
Starting off with my rpi 3 B+.

# Set up cross compilation
I'll be compiling on my linux box and moving the executable(s) to the pi.

```bash
rustup target add aarch64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-musl
```
