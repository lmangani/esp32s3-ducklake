# ESP32-S3 DuckDB DuckLake S3 Writer

Experimental proof-of-concept for **opensensor.space** demonstrating how to use DuckDB with DuckLake extension on ESP32-S3 to write sensor data directly to S3.

## Features

- **DuckDB DuckLake**: Uses DuckDB with DuckLake extension for data lake management
- **S3 Integration**: Automatically writes Parquet files to AWS S3 with DuckLake metadata management
- **ACID Transactions**: DuckLake provides ACID guarantees and time travel queries
- **Schema Evolution**: Support for schema changes over time
- **Automatic Compression**: DuckDB handles Parquet compression automatically
- **Time Sync**: Synchronizes time via SNTP (NTP) for S3 authentication

## Hardware

- **Device**: ESP32-S3 (Required for sufficient PSRAM and memory for DuckDB)
- **Storage**: Uses DuckDB in-memory database with DuckLake managing S3 storage
- **Note**: DuckDB may have higher memory/binary size requirements than raw Parquet

## Dependencies

- [`duckdb`](https://crates.io/crates/duckdb): DuckDB Rust client (v1.4+)
- [`esp-idf-svc`](https://crates.io/crates/esp-idf-svc): ESP-IDF service wrappers (WiFi, HTTP, SNTP)
- [`rusty-s3`](https://crates.io/crates/rusty-s3): May still be needed for some S3 operations (kept for compatibility)

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

1.  Connects to WiFi (required for DuckLake S3 operations).
2.  Synchronizes time via NTP (crucial for AWS authentication).
3.  Initializes DuckDB connection and loads DuckLake extension.
4.  Configures S3 credentials and attaches DuckLake with S3 storage path.
5.  Creates sensor readings table in DuckLake.
6.  Inserts sensor data batches (DuckLake automatically writes Parquet files to S3).
7.  Verifies data with queries.

## DuckLake Benefits

- **ACID Transactions**: Data consistency guarantees
- **Time Travel**: Query historical snapshots of data
- **Schema Evolution**: Add/modify columns without breaking existing data
- **Automatic File Management**: DuckLake handles Parquet file organization on S3
- **Better Query Capabilities**: Full SQL support vs. raw Parquet files

## Important Notes

⚠️ **Memory Considerations**: DuckDB is a full database engine and may require more memory than the previous Parquet-only approach. Monitor memory usage carefully.

⚠️ **Binary Size**: DuckDB binary may be larger than the `parquet` crate. Ensure sufficient flash space.

⚠️ **Network Required**: DuckLake requires network connectivity for S3 operations. No offline mode available.

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

### Running CI Locally

To test the build process locally:

```bash
# Install ESP toolchain
cargo install espup
espup install

# Source the environment
source ~/export-esp.sh

# Build the project
cargo build --release --target xtensa-esp32s3-espidf

# Check binary size
ls -lh target/xtensa-esp32s3-espidf/release/esp32s3-parquet-test
```

### QEMU Testing

⚠️ **Note**: Full ESP32-S3 QEMU simulation support is experimental. The CI workflow validates binary format and structure, but full QEMU execution may require additional setup. For complete testing, flash to actual ESP32-S3 hardware.
