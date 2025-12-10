# ESP32-S3 Parquet S3 Uploader

Experimental proof-of-concept for **opensensor.space** demonstrating how to generate Snappy-compressed Parquet files directly on an ESP32-S3 and upload them to AWS S3.

## Features

- **Parquet Generation**: Creates legitimate Parquet files with sensor schema (Timestamp, Temp, Humidity, PM2.5, etc.) directly on the microcontroller.
- **Compression**: Uses **Snappy** compression (standard for Parquet) to minimize data size (~7-8KB for 178 rows).
- **S3 Upload**: Uploads files to AWS S3 using **Chunked Transfer Encoding** (8KB chunks) via HTTP PUT.
- **Security**: Generates AWS Signature V4 presigned URLs on-device using `rusty-s3`.
- **Time Sync**: Synchronizes time via SNTP (NTP) to ensure valid AWS signatures.

## Hardware

- **Device**: ESP32-S3 (Required for sufficient PSRAM, though this experiment runs comfortably in <200KB RAM).
- **Storage**: Uses internal PSRAM/Heap for file buffering (no SD card required for this demo).

## Dependencies

- [`parquet`](https://crates.io/crates/parquet): The official Rust Parquet implementation.
- [`esp-idf-svc`](https://crates.io/crates/esp-idf-svc): ESP-IDF service wrappers (WiFi, HTTP, SNTP).
- [`rusty-s3`](https://crates.io/crates/rusty-s3): Pure Rust, Sans-IO S3 client for signing requests.

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

1.  Connects to WiFi.
2.  Synchronizes time via NTP (crucial for AWS authentication).
3.  Generates synthetic sensor data (simulating 15 minutes of 5-second samples).
4.  Writes data to an in-memory Parquet writer with Snappy compression.
5.  Calculates the AWS V4 signature and streams the binary data to S3 using chunked HTTP requests.
