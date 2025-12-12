//! ESP32-C6 DuckDB DuckLake + S3 Experiment
//!
//! This experimental code for opensensor.space demonstrates:
//! 1. Using DuckDB with DuckLake extension on ESP32-C6 (RISC-V)
//! 2. Writing sensor data to DuckLake tables stored on S3
//! 3. Testing with 3 sample sensor data batches
//!
//! IMPORTANT: Replace AWS credentials and WiFi settings before flashing!
//!
//! NOTE: ESP32-C6 uses RISC-V architecture which has better compatibility
//! with DuckDB/Arrow compared to Xtensa-based ESP32 chips.

use std::time::Duration;

use anyhow::{bail, Result};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::{error, info, warn};
use duckdb::{Connection, params};

// ============================================================================
// CONFIGURATION - REPLACE THESE VALUES!
// ============================================================================

// WiFi Configuration
const WIFI_SSID: &str = "YOUR_WIFI";
const WIFI_PASSWORD: &str = "YOUR_PASSWORD";

// AWS S3 Configuration for DuckLake
const AWS_ACCESS_KEY: &str = "YOUR_ACCESS_KEY";
const AWS_SECRET_KEY: &str = "YOUR_SECRET_KEY";
const S3_BUCKET: &str = "YOUR_BUCKET";
const S3_REGION: &str = "us-west-2";
const S3_ENDPOINT: &str = ""; // Leave empty for AWS S3, or set custom endpoint

// DuckLake Configuration
const DUCKLAKE_NAME: &str = "sensor_data_lake";
const TABLE_NAME: &str = "sensor_readings";

// Test settings
const NUM_TEST_BATCHES: usize = 3;
const ROWS_PER_BATCH: usize = 178; // Similar to opensensor.space data

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("================================================");
    info!("ESP32-C6 DuckDB DuckLake + S3 Experiment");
    info!("For opensensor.space");
    info!("================================================");

    // Initialize peripherals
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Connect to WiFi
    info!("Step 1: Connecting to WiFi...");
    let _wifi = match connect_wifi(peripherals.modem, sys_loop, nvs) {
        Ok(wifi) => {
            info!("WiFi connected successfully!");
            wifi
        }
        Err(e) => {
            error!("WiFi connection failed: {:?}", e);
            error!("DuckLake requires network connectivity for S3 access");
            error!("Cannot run in offline mode");
            return Err(e.into());
        }
    };

    // Synchronize time (required for S3 authentication)
    if let Err(e) = initialize_sntp() {
        error!("Failed to synchronize time: {:?}", e);
        warn!("Continuing anyway, but S3 operations may fail");
    }

    // Run the full experiment with DuckLake
    run_ducklake_experiment()?;

    info!("================================================");
    info!("Experiment complete!");
    info!("================================================");

    // Keep running (don't exit)
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}

// ============================================================================
// WIFI CONNECTION
// ============================================================================

fn connect_wifi(
    modem: esp_idf_svc::hal::modem::Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'static>>> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        password: WIFI_PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;

    info!("WiFi started, connecting to '{}'...", WIFI_SSID);
    wifi.connect()?;

    info!("Waiting for DHCP...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi connected! IP: {}", ip_info.ip);

    Ok(wifi)
}

// ============================================================================
// SNTP TIME SYNC
// ============================================================================

fn initialize_sntp() -> Result<()> {
    info!("Step 1.5: Synchronizing time via SNTP...");

    let sntp = esp_idf_svc::sntp::EspSntp::new_default()?;
    info!("SNTP initialized, waiting for status sync...");

    let mut wait_count = 0;
    while sntp.get_sync_status() != esp_idf_svc::sntp::SyncStatus::Completed {
        std::thread::sleep(Duration::from_millis(100));
        wait_count += 1;
        
        // Print progress every second
        if wait_count % 10 == 0 {
            info!("  Waiting for time sync... ({}s)", wait_count / 10);
        }

        // Timeout after 15 seconds
        if wait_count > 150 {
             bail!("Timeout waiting for SNTP time sync");
        }
    }

    // Log current time
    let now = std::time::SystemTime::now();
    let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    info!("Time synchronized! Unix timestamp: {}", since_epoch);

    Ok(())
}

// ============================================================================
// DUCKLAKE EXPERIMENT WITH S3
// ============================================================================

fn run_ducklake_experiment() -> Result<()> {
    info!("Step 2: Setting up DuckDB with DuckLake extension...");

    // Create in-memory DuckDB connection
    let conn = Connection::open_in_memory()?;
    info!("DuckDB connection established");

    // Install and load DuckLake extension
    info!("Installing DuckLake extension...");
    conn.execute("INSTALL ducklake;", [])?;
    conn.execute("LOAD ducklake;", [])?;
    info!("DuckLake extension loaded");

    // Configure S3 credentials for DuckDB
    info!("Configuring S3 credentials...");
    conn.execute(
        &format!("SET s3_region='{}';", S3_REGION),
        [],
    )?;
    conn.execute(
        &format!("SET s3_access_key_id='{}';", AWS_ACCESS_KEY),
        [],
    )?;
    conn.execute(
        &format!("SET s3_secret_access_key='{}';", AWS_SECRET_KEY),
        [],
    )?;
    
    if !S3_ENDPOINT.is_empty() {
        conn.execute(
            &format!("SET s3_endpoint='{}';", S3_ENDPOINT),
            [],
        )?;
    }
    info!("S3 credentials configured");

    // Attach DuckLake with S3 storage
    // DuckLake will store metadata in a local file and data files in S3
    // Note: The metadata file (.ducklake) will be created in the current directory
    // For production, consider storing metadata in NVS or a persistent filesystem
    let s3_data_path = format!("s3://{}/opensensor-test/esp32s3/ducklake-data", S3_BUCKET);
    let attach_sql = format!(
        "ATTACH 'ducklake:{}.ducklake' AS {} (DATA_PATH '{}');",
        DUCKLAKE_NAME, DUCKLAKE_NAME, s3_data_path
    );
    
    info!("Attaching DuckLake: {}", attach_sql);
    conn.execute(&attach_sql, [])?;
    info!("DuckLake attached successfully");

    // Switch to DuckLake database
    conn.execute(&format!("USE {};", DUCKLAKE_NAME), [])?;

    // Create sensor readings table
    info!("Creating table: {}", TABLE_NAME);
    let create_table_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            timestamp BIGINT NOT NULL,
            temperature REAL NOT NULL,
            humidity REAL NOT NULL,
            pressure REAL NOT NULL,
            pm1_0 REAL NOT NULL,
            pm2_5 REAL NOT NULL,
            pm10 REAL NOT NULL,
            gas_resistance REAL NOT NULL,
            light REAL NOT NULL,
            noise REAL NOT NULL
        );",
        TABLE_NAME
    );
    conn.execute(&create_table_sql, [])?;
    info!("Table created successfully");

    // Insert test data batches
    info!("Inserting {} batches of sensor data...", NUM_TEST_BATCHES);
    let mut total_rows_inserted = 0;

    for batch_idx in 0..NUM_TEST_BATCHES {
        info!("----------------------------------------");
        info!("Processing batch {}/{}...", batch_idx + 1, NUM_TEST_BATCHES);

        // Generate sensor data for this batch
        let sensor_data = generate_sensor_data(batch_idx as u64)?;
        
        // Insert data using prepared statement for efficiency
        let insert_sql = format!(
            "INSERT INTO {} (timestamp, temperature, humidity, pressure, pm1_0, pm2_5, pm10, gas_resistance, light, noise) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
            TABLE_NAME
        );
        let mut stmt = conn.prepare(&insert_sql)?;

        for row in &sensor_data {
            stmt.execute(params![
                row.timestamp,
                row.temperature,
                row.humidity,
                row.pressure,
                row.pm1_0,
                row.pm2_5,
                row.pm10,
                row.gas_resistance,
                row.light,
                row.noise
            ])?;
        }

        total_rows_inserted += sensor_data.len();
        info!("  Batch {} inserted: {} rows", batch_idx + 1, sensor_data.len());
    }

    // Query to verify data
    info!("----------------------------------------");
    info!("Verifying data...");
    let mut stmt = conn.prepare(&format!("SELECT COUNT(*) FROM {};", TABLE_NAME))?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    info!("Total rows in table: {}", count);

    // Show sample data
    let mut stmt = conn.prepare(&format!("SELECT * FROM {} LIMIT 3;", TABLE_NAME))?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,  // timestamp
            row.get::<_, f32>(1)?,  // temperature
            row.get::<_, f32>(2)?,  // humidity
        ))
    })?;

    info!("Sample rows:");
    for (idx, row_result) in rows.enumerate() {
        let (ts, temp, hum) = row_result?;
        info!("  Row {}: timestamp={}, temp={:.2}, humidity={:.2}", idx + 1, ts, temp, hum);
    }

    info!("----------------------------------------");
    info!("DuckLake Experiment Summary:");
    info!("  Batches processed: {}", NUM_TEST_BATCHES);
    info!("  Total rows inserted: {}", total_rows_inserted);
    info!("  Table: {}.{}", DUCKLAKE_NAME, TABLE_NAME);
    info!("  S3 location: {}", s3_data_path);

    Ok(())
}

// ============================================================================
// SENSOR DATA STRUCTURE AND GENERATION
// ============================================================================

#[derive(Debug, Clone)]
struct SensorReading {
    timestamp: i64,
    temperature: f32,
    humidity: f32,
    pressure: f32,
    pm1_0: f32,
    pm2_5: f32,
    pm10: f32,
    gas_resistance: f32,
    light: f32,
    noise: f32,
}

fn generate_sensor_data(batch_index: u64) -> Result<Vec<SensorReading>> {
    // Base timestamp (simulate different time windows per batch)
    let base_timestamp = 1733270400000i64 + (batch_index as i64 * 900000); // 15 min apart

    let mut readings = Vec::with_capacity(ROWS_PER_BATCH);

    for i in 0..ROWS_PER_BATCH {
        let timestamp = base_timestamp + (i as i64 * 5000); // 5 second intervals

        readings.push(SensorReading {
            timestamp,
            temperature: 20.0 + (i as f32 * 0.02) + (batch_index as f32 * 0.5),
            humidity: 45.0 + (i as f32 * 0.05) + (batch_index as f32 * 2.0),
            pressure: 1013.25 + (i as f32 * 0.01),
            pm1_0: 5.0 + (i as f32 % 10.0) * 0.1,
            pm2_5: 8.0 + (i as f32 % 15.0) * 0.2,
            pm10: 12.0 + (i as f32 % 20.0) * 0.3,
            gas_resistance: 50000.0 + (i as f32 * 100.0),
            light: 100.0 + (i as f32 * 2.0),
            noise: 35.0 + (i as f32 % 10.0) * 0.5,
        });
    }

    Ok(readings)
}


// ============================================================================
// NOTES FOR OPENSENSOR.SPACE INTEGRATION
// ============================================================================
//
// This experimental code demonstrates:
//
// 1. DuckDB with DuckLake extension on ESP32-S3
//    - Uses DuckDB's built-in Parquet writing capabilities
//    - DuckLake manages metadata and data files on S3
//    - Automatic compression and optimization
//
// 2. DuckLake benefits:
//    - ACID transactions and time travel queries
//    - Schema evolution support
//    - Automatic file management on S3
//    - Better query capabilities than raw Parquet files
//
// 3. Important considerations:
//    - DuckDB may have higher memory/binary size requirements than raw Parquet
//    - Requires network connectivity (no offline mode)
//    - DuckLake extension must be installed and loaded
//    - S3 credentials configured via DuckDB settings
//
// For production:
// - Monitor memory usage (DuckDB may use more than raw Parquet)
// - Consider binary size constraints (DuckDB is larger than parquet crate)
// - Add retry logic for network operations
// - Use secure credential storage (NVS encrypted partition)
// - Test DuckLake maintenance operations (merge files, expire snapshots)
// - Consider partitioning strategies for large datasets
//
