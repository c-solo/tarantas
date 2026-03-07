# Tarantas Robot

Rust-based robot control system.

- **`engine/lib`** - STM32 firmware core library (drivers, system, bus)
- **`engine/firmware`** - STM32 application entry point
- **`control`** - Jetson Orin Nano control software
- **`protocol`** - Communication protocol between `control` and `engine`

## Build

Each package has its own target configuration in `.cargo/config.toml`:
- `engine/lib` and `engine/firmware` → `thumbv7em-none-eabihf` (STM32)
- `control` → host target (Jetson Orin Nano) or can be overridden

**STM32 firmware:**
```bash
# From package directory (uses local .cargo/config.toml):
cd engine/firmware
cargo build --release

# From workspace root (also uses package config):
cargo build -p firmware --release
```

**Jetson Orin Nano control:**
```bash
# From package directory (uses host target):
cd control
cargo build --release

# From workspace root:
cargo build -p control --release

# For cross-compilation to aarch64 (override local config):
cargo build --target aarch64-unknown-linux-gnu -p control --release
```

## Flash

```bash
cd engine/firmware
DEFMT_LOG=info cargo run --release
# or from root:
DEFMT_LOG=info cargo run -p firmware --release
```
