# brotkasten

brotkasten is super simple LAN radio station and receiver demo/POC sending raw audio frames via udp broadcast.

## Transmitter

The transmitter collects samples from the first audio input device found by `cpal` and broadcasts its samples on `0.0.0.0:12345`.

```
$ cd transmitter && cargo run --release
```

## Receiver

receiver collects samples on `0.0.0.0:12345` and plays them on the first audio output device that it finds.

```
$ cd receiver && cargo run --release
```

## Configuration

Just edit `config.rs` and rebuild.
