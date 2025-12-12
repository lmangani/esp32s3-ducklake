# ESP32-S3 Parquet S3 Writer

Experimental proof-of-concept for **opensensor.space** demonstrating how to create Snappy-compressed Parquet files on ESP32-S3 and upload them to AWS S3 using chunked transfer encoding.

## Features

- **Parquet Files**: Creates Snappy-compressed Parquet files with sensor data
- **S3 Upload**: Uploads Parquet files to AWS S3 using presigned URLs and chunked transfer
- **Offline Mode**: Can create Parquet files without network connectivity
- **Time Sync**: Synchronizes time via SNTP (NTP) for S3 authentication
- **Xtensa Architecture**: Uses ESP32-S3 (Xtensa) - proven to work with parquet crate

## Hardware

- **Device**: ESP32-S3 (Xtensa architecture)
- **Storage**: In-memory Parquet file creation, then upload to S3
- **Note**: Binary size ~997KB (24.73% of 4MB partition)

## Dependencies

- [`parquet`](https://crates.io/crates/parquet): Parquet file format support (v56.x, with `snap` feature for Snappy compression)
- [`esp-idf-svc`](https://crates.io/crates/esp-idf-svc): ESP-IDF service wrappers (WiFi, HTTP, SNTP)
- [`rusty-s3`](https://crates.io/crates/rusty-s3): Sans-IO S3 client for generating presigned URLs

## Setup & Usage

1.  **Configure Credentials**:
    Open `src/main.rs` and update the configuration section with your details:

    ```rust
    const WIFI_SSID: &str = "YOUR_WIFI";
    const WIFI_PASSWORD: &str = "YOUR_PASSWORD";
    const AWS_ACCESS_KEY: &str = "YOUR_AWS_KEY";
    const AWS_SECRET_KEY: &str = "YOUR_AWS_SECRET";
    const S3_BUCKET: &str = "your-bucket-name";
    ```

2.  **Build & Flash**:

    ```bash
    # Ensure ESP environment is sourced
    . ~/export-esp.sh

    # Build and flash (release mode is recommended for Snappy performance)
    cargo build --release
    espflash flash --monitor
    ```

## How It Works

1.  Connects to WiFi (optional - can run in offline mode).
2.  Synchronizes time via NTP (required for AWS S3 authentication).
3.  Creates Snappy-compressed Parquet files in memory with sensor data.
4.  Generates presigned S3 URLs using `rusty-s3`.
5.  Uploads Parquet files to S3 using chunked transfer encoding via `esp-idf-svc` HTTP client.
6.  Verifies upload success.

## Parquet File Structure

Each Parquet file contains:
- **178 rows** of sensor data (similar to opensensor.space)
- **10 columns**: timestamp, temperature, humidity, pressure, pm1_0, pm2_5, pm10, gas_resistance, light, noise
- **Compression**: Snappy (pure Rust implementation)
- **File size**: ~10-15KB per file

## Important Notes

✅ **Arrow-Buffer Xtensa Support**: The `parquet` crate v56 depends on `arrow-buffer`, which doesn't support Xtensa architecture by default. This project includes a patched version of `arrow-rs` as a git submodule that adds Xtensa support.

**Initial Setup:**
```bash
# Clone the repository with submodules
git clone --recursive <repository-url>

# Or if you already cloned without --recursive:
git submodule update --init --recursive
```

**Submodule Details:**
- Location: `vendor/arrow-rs/`
- Patch: Adds Xtensa architecture support to `arrow-buffer/src/alloc/alignment.rs`
- The patch is automatically applied via `[patch.crates-io]` in `Cargo.toml`

✅ **Memory Efficient**: Uses minimal memory compared to database engines.

✅ **Offline Capable**: Can create Parquet files without network connectivity.

⚠️ **Network Required for Upload**: S3 upload requires WiFi connectivity and time synchronization.

## CI/CD with GitHub Actions

This project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that:

- **Builds** the project for ESP32-S3 in both debug and release modes
- **Validates** binary format and size
- **Converts** ELF binaries to flashable format using esptool
- **Tests** binary structure for QEMU compatibility (ESP32-S3 QEMU support is experimental)
- **Lints** code with rustfmt and clippy
- **Caches** dependencies and build artifacts for faster CI runs

### CI Workflow Features

- Automatic builds on push/PR to main/master/develop branches
- Binary size warnings (alerts if > 3MB)
- Artifact uploads for easy binary access
- Build summary with status of all jobs
- Short-term caching for Rust builds to save action time

### Running CI Locally

To test the build process locally:

```bash
# Install ESP toolchain
cargo install espup
espup install --targets esp32s3

# Source the environment
source ~/export-esp.sh

# Build the project
cargo build --release --target xtensa-esp32s3-espidf

# Check binary size
ls -lh target/xtensa-esp32s3-espidf/release/esp32s3-parquet-test
```

### QEMU Testing

⚠️ **Note**: Full ESP32-S3 QEMU simulation support is experimental. The CI workflow validates binary format and structure, but full QEMU execution may require additional setup. For complete testing, flash to actual ESP32-S3 hardware.

## Why ESP32-S3 (Xtensa)?

ESP32-S3 uses the Xtensa architecture and is the proven working target for this project:
- ✅ **Proven Compatibility**: The parquet crate works reliably on Xtensa
- ✅ **8MB PSRAM**: Ample memory for Parquet file creation
- ✅ **Stable Toolchain**: Well-supported ESP-IDF toolchain
- ⚠️ **Note**: ESP32-C6 (RISC-V) has compatibility issues with `arrow-buffer` dependency in parquet crate

## Future Enhancements

- Add retry logic with exponential backoff for S3 uploads
- Implement multipart upload for files > 5MB (unlikely with sensor data)
- Add Hive-style partitioning: `s3://bucket/station=DEVICE_ID/year=YYYY/month=MM/day=DD/data_HHMM.parquet`
- Use secure credential storage (NVS encrypted partition)
- Add compression ratio vs. CPU trade-off analysis
