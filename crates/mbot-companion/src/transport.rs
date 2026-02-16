//! Transport layer for mBot2 communication via CyberPi HalocodeProtocol (f3/f4)
//!
//! Uses the correct protocol for CyberPi: sends Python expressions inside f3 frames,
//! CyberPi evaluates them and returns JSON results. This is how mBlock "Live Mode" works.
//! Works over both USB Serial and Bluetooth Low Energy.

#[cfg(feature = "bluetooth")]
use anyhow::anyhow;
use anyhow::{Context, Result};
use mbot_core::{MBotSensors, MotorCommand};
#[cfg(feature = "serial")]
use std::io::Write as _;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::protocol;

pub enum TransportType {
    #[cfg(feature = "bluetooth")]
    Bluetooth,
    #[cfg(feature = "serial")]
    Serial(String),
    Simulated,
}

/// Tracks which sensors responded successfully on the last read cycle.
#[derive(Clone, Debug, Default)]
pub struct SensorHealth {
    pub brightness_ok: bool,
    pub loudness_ok: bool,
    pub gyro_ok: bool,
    pub accel_ok: bool,
    pub ultrasonic_ok: bool,
}

impl SensorHealth {
    pub fn ok_count(&self) -> usize {
        [self.brightness_ok, self.loudness_ok, self.gyro_ok, self.accel_ok, self.ultrasonic_ok]
            .iter()
            .filter(|&&v| v)
            .count()
    }

    pub fn total(&self) -> usize {
        5
    }

    pub fn failed_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if !self.brightness_ok { names.push("Brightness"); }
        if !self.loudness_ok { names.push("Loudness"); }
        if !self.gyro_ok { names.push("Gyro"); }
        if !self.accel_ok { names.push("Accel"); }
        if !self.ultrasonic_ok { names.push("Ultrasonic"); }
        names
    }
}

/// CyberPi BLE UUIDs (Makeblock standard)
#[cfg(feature = "bluetooth")]
mod ble_uuids {
    use uuid::Uuid;
    pub const SERVICE: Uuid = Uuid::from_u128(0x0000ffe1_0000_1000_8000_00805f9b34fb);
    pub const NOTIFY_CHAR: Uuid = Uuid::from_u128(0x0000ffe2_0000_1000_8000_00805f9b34fb);
    pub const WRITE_CHAR: Uuid = Uuid::from_u128(0x0000ffe3_0000_1000_8000_00805f9b34fb);
}

pub struct MBotTransport {
    inner: TransportInner,
    // Simulation state
    sim_distance: f32,
    sim_encoder_left: i32,
    sim_encoder_right: i32,
    sim_tick: u64,
    // Last known good sensor values (used when a sensor read fails)
    last_sensors: MBotSensors,
    // Which sensors succeeded on the last read cycle
    sensor_health: SensorHealth,
    // f3 protocol state (shared by serial and bluetooth)
    parser: protocol::F3Parser,
    next_idx: u16,
}

enum TransportInner {
    #[cfg(feature = "bluetooth")]
    Bluetooth(BluetoothTransport),
    #[cfg(feature = "serial")]
    Serial(SerialTransport),
    Simulated,
}

#[cfg(feature = "bluetooth")]
struct BluetoothTransport {
    peripheral: btleplug::platform::Peripheral,
    write_char: btleplug::api::Characteristic,
    _notify_char: btleplug::api::Characteristic,
}

#[cfg(feature = "serial")]
struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl MBotTransport {
    pub async fn connect(transport_type: TransportType) -> Result<Self> {
        let inner = match transport_type {
            #[cfg(feature = "bluetooth")]
            TransportType::Bluetooth => {
                let bt = Self::connect_bluetooth().await?;
                TransportInner::Bluetooth(bt)
            }
            #[cfg(feature = "serial")]
            TransportType::Serial(port_name) => {
                let serial = Self::connect_serial(&port_name)?;
                TransportInner::Serial(serial)
            }
            TransportType::Simulated => TransportInner::Simulated,
        };

        let mut transport = Self {
            inner,
            sim_distance: 100.0,
            sim_encoder_left: 0,
            sim_encoder_right: 0,
            sim_tick: 0,
            last_sensors: MBotSensors::default(),
            sensor_health: SensorHealth::default(),
            parser: protocol::F3Parser::new(),
            next_idx: 1,
        };

        // Initialize CyberPi protocol
        #[cfg(feature = "serial")]
        if matches!(&transport.inner, TransportInner::Serial(_)) {
            transport.init_cyberpi_protocol()?;
        }
        #[cfg(feature = "bluetooth")]
        if matches!(&transport.inner, TransportInner::Bluetooth(_)) {
            transport.init_cyberpi_protocol_ble().await?;
        }

        Ok(transport)
    }

    pub fn sensor_health(&self) -> &SensorHealth {
        &self.sensor_health
    }

    /// Get the next packet index, wrapping at 0xffff.
    fn next_idx(&mut self) -> u16 {
        let idx = self.next_idx;
        self.next_idx = self.next_idx.wrapping_add(1);
        if self.next_idx == 0 { self.next_idx = 1; }
        idx
    }

    // =========================================================================
    //  SERIAL transport
    // =========================================================================

    /// Initialize the CyberPi HalocodeProtocol over serial.
    /// ESP32 boot takes ~6 seconds. We wait for boot text, then broadcast.
    #[cfg(feature = "serial")]
    fn init_cyberpi_protocol(&mut self) -> Result<()> {
        info!("Initializing CyberPi HalocodeProtocol (serial)...");

        // Phase 1: Wait for ESP32 boot to complete.
        info!("Waiting for CyberPi boot...");
        {
            let port = match &mut self.inner {
                TransportInner::Serial(s) => &mut s.port,
                _ => return Ok(()),
            };

            let boot_deadline = Instant::now() + Duration::from_secs(8);
            let mut boot_done = false;
            let mut buf = [0u8; 512];

            while Instant::now() < boot_deadline {
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let text = String::from_utf8_lossy(&buf[..n]);
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            debug!("BOOT: {}", trimmed);
                        }
                        if text.contains("wifi") || text.contains("ESPNOW") {
                            boot_done = true;
                            std::thread::sleep(Duration::from_millis(500));
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => {}
                }
            }

            if boot_done {
                info!("CyberPi boot complete.");
            } else {
                info!("Boot timeout - CyberPi may already be running. Continuing...");
            }

            let _ = port.clear(serialport::ClearBuffer::Input);

            let online = protocol::online_mode_frame();
            debug!("TX online_mode: {:02X?}", online);
            port.write_all(&online)?;
            port.flush()?;

            std::thread::sleep(Duration::from_millis(200));
            let _ = port.clear(serialport::ClearBuffer::Input);
        }

        // Phase 2: Broadcast until CyberPi responds with an f3 frame.
        info!("Sending broadcast to CyberPi...");
        let mut ready = false;
        for attempt in 0..10 {
            let idx = self.next_idx();
            let frame = protocol::sensor_read(protocol::read_brightness_script(), idx);

            let port = match &mut self.inner {
                TransportInner::Serial(s) => &mut s.port,
                _ => return Ok(()),
            };

            debug!("TX broadcast[{}] idx={}", attempt, idx);

            if let Err(e) = port.write_all(&frame) {
                warn!("TX failed: {}", e);
                continue;
            }
            let _ = port.flush();

            let wait_deadline = Instant::now() + Duration::from_millis(500);
            let mut buf = [0u8; 512];
            while Instant::now() < wait_deadline {
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let responses = self.parser.feed(&buf[..n]);
                        if !responses.is_empty() {
                            info!("CyberPi responded! Protocol ready. (attempt {})", attempt + 1);
                            ready = true;
                            break;
                        }
                        let ascii = String::from_utf8_lossy(&buf[..n]);
                        let trimmed = ascii.trim();
                        if !trimmed.is_empty() {
                            debug!("RX (text): {}", trimmed);
                        }
                    }
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        debug!("RX error: {}", e);
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            if ready { break; }
        }

        if !ready {
            warn!("CyberPi did not respond to broadcast after 10 attempts.");
            warn!("Sensor reads may fail. Make sure CyberPi is powered on and connected.");
        }

        // Phase 3: Drain stale responses.
        {
            let port = match &mut self.inner {
                TransportInner::Serial(s) => &mut s.port,
                _ => return Ok(()),
            };

            let drain_deadline = Instant::now() + Duration::from_millis(300);
            let mut buf = [0u8; 512];
            let mut drained = 0usize;
            while Instant::now() < drain_deadline {
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let responses = self.parser.feed(&buf[..n]);
                        drained += responses.len();
                    }
                    _ => break,
                }
            }
            if drained > 0 {
                debug!("Drained {} stale responses after init", drained);
            }
        }

        self.next_idx = 1;
        Ok(())
    }

    #[cfg(feature = "serial")]
    fn connect_serial(port_name: &str) -> Result<SerialTransport> {
        info!("Opening serial port: {}", port_name);

        let mut port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(200))
            .open()
            .context(format!("Failed to open serial port: {}", port_name))?;

        if let Err(e) = port.write_data_terminal_ready(false) {
            debug!("Could not set DTR=false: {}", e);
        }
        if let Err(e) = port.write_request_to_send(false) {
            debug!("Could not set RTS=false: {}", e);
        }

        info!("Serial port opened (DTR=false, RTS=false)");
        Ok(SerialTransport { port })
    }

    /// Send an f3 script frame over serial and wait for matching response.
    #[cfg(feature = "serial")]
    fn query_sensor_serial(&mut self, script: &str, timeout_ms: u64) -> Option<protocol::CyberPiValue> {
        let idx = self.next_idx();
        let frame = protocol::sensor_read(script, idx);

        debug!("TX [idx={}]: {} ({} bytes)", idx, script, frame.len());

        {
            let port = match &mut self.inner {
                TransportInner::Serial(s) => &mut s.port,
                _ => return None,
            };

            if let Err(e) = port.write_all(&frame) {
                debug!("TX failed: {}", e);
                return None;
            }
            let _ = port.flush();
        }

        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let mut buf = [0u8; 512];

        while Instant::now() < deadline {
            let port = match &mut self.inner {
                TransportInner::Serial(s) => &mut s.port,
                _ => return None,
            };

            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let responses = self.parser.feed(&buf[..n]);
                    for resp in responses {
                        if resp.idx == idx && resp.frame_type == protocol::TYPE_SCRIPT {
                            debug!("RX [idx={}]: {:?} ({})", resp.idx, resp.value, resp.raw_json);
                            return Some(resp.value);
                        }
                        debug!("RX [idx={} != {}]: {:?}", resp.idx, idx, resp.value);
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    debug!("RX error: {}", e);
                    return None;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        debug!("RX timeout for idx={}", idx);
        None
    }

    /// Send a fire-and-forget f3 command over serial.
    #[cfg(feature = "serial")]
    fn send_f3_command_serial(&mut self, script: &str) -> bool {
        let idx = self.next_idx();
        let frame = protocol::command(script, idx);

        debug!("TX cmd [idx={}]: {} ({} bytes)", idx, script, frame.len());

        let port = match &mut self.inner {
            TransportInner::Serial(s) => &mut s.port,
            _ => return false,
        };

        match port.write_all(&frame) {
            Ok(()) => {
                let _ = port.flush();
                true
            }
            Err(e) => {
                debug!("TX failed: {}", e);
                false
            }
        }
    }

    /// Read all sensors over serial.
    #[cfg(feature = "serial")]
    fn read_sensors_serial(&mut self) -> Result<MBotSensors> {
        let mut sensors = self.last_sensors.clone();
        let mut health = SensorHealth::default();

        const INTER_QUERY_MS: u64 = 30;
        const TIMEOUT_MS: u64 = 500;

        if let Some(val) = self.query_sensor_serial(protocol::read_brightness_script(), TIMEOUT_MS) {
            if let Some(v) = val.as_f32() {
                sensors.light_level = (v / 100.0).clamp(0.0, 1.0);
                health.brightness_ok = true;
            }
        }
        std::thread::sleep(Duration::from_millis(INTER_QUERY_MS));

        if let Some(val) = self.query_sensor_serial(protocol::read_loudness_script(), TIMEOUT_MS) {
            if let Some(v) = val.as_f32() {
                sensors.sound_level = (v / 100.0).clamp(0.0, 1.0);
                health.loudness_ok = true;
            }
        }
        std::thread::sleep(Duration::from_millis(INTER_QUERY_MS));

        if let Some(val) = self.query_sensor_serial(&protocol::read_gyro_script('z'), TIMEOUT_MS) {
            if let Some(v) = val.as_f32() {
                sensors.gyro_z = v;
                health.gyro_ok = true;
            }
        }
        std::thread::sleep(Duration::from_millis(INTER_QUERY_MS));

        let mut accel_ok = false;
        for (i, axis) in ['x', 'y', 'z'].iter().enumerate() {
            if let Some(val) = self.query_sensor_serial(&protocol::read_accel_script(*axis), TIMEOUT_MS) {
                if let Some(v) = val.as_f32() {
                    sensors.accel[i] = v;
                    accel_ok = true;
                }
            }
            if i < 2 {
                std::thread::sleep(Duration::from_millis(INTER_QUERY_MS));
            }
        }
        health.accel_ok = accel_ok;
        std::thread::sleep(Duration::from_millis(INTER_QUERY_MS));

        if let Some(val) = self.query_sensor_serial(protocol::read_ultrasonic_script(), TIMEOUT_MS) {
            if let Some(v) = val.as_f32() {
                sensors.ultrasonic_cm = v;
                health.ultrasonic_ok = true;
            }
        }

        sensors.timestamp_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        self.last_sensors = sensors.clone();
        self.sensor_health = health;

        Ok(sensors)
    }

    // =========================================================================
    //  BLUETOOTH transport
    // =========================================================================

    #[cfg(feature = "bluetooth")]
    async fn connect_bluetooth() -> Result<BluetoothTransport> {
        use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
        use btleplug::platform::Manager;

        info!("Scanning for mBot2 via BLE...");

        let manager = Manager::new()
            .await
            .context("Failed to create Bluetooth manager")?;

        let adapters = manager
            .adapters()
            .await
            .context("Failed to get Bluetooth adapters")?;

        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No Bluetooth adapter found"))?;

        adapter
            .start_scan(ScanFilter::default())
            .await
            .context("Failed to start Bluetooth scan")?;

        // Scan for 5 seconds
        tokio::time::sleep(Duration::from_secs(5)).await;

        let peripherals = adapter
            .peripherals()
            .await
            .context("Failed to get peripherals")?;

        for peripheral in peripherals {
            if let Ok(Some(props)) = peripheral.properties().await {
                let name = props.local_name.unwrap_or_default();
                debug!("Found BLE device: {}", name);

                if name.contains("Makeblock") || name.contains("CyberPi") || name.contains("mBot") {
                    info!("Found mBot2: {}", name);

                    peripheral
                        .connect()
                        .await
                        .context("Failed to connect to mBot2")?;

                    peripheral
                        .discover_services()
                        .await
                        .context("Failed to discover services")?;

                    let chars = peripheral.characteristics();

                    let write_char = chars
                        .iter()
                        .find(|c| c.uuid == ble_uuids::WRITE_CHAR)
                        .cloned()
                        .ok_or_else(|| anyhow!("Write characteristic (ffe3) not found"))?;

                    let notify_char = chars
                        .iter()
                        .find(|c| c.uuid == ble_uuids::NOTIFY_CHAR)
                        .cloned()
                        .ok_or_else(|| anyhow!("Notify characteristic (ffe2) not found"))?;

                    // Subscribe to notifications
                    peripheral
                        .subscribe(&notify_char)
                        .await
                        .context("Failed to subscribe to BLE notifications")?;

                    info!("Connected to mBot2 via BLE! (ffe2 subscribed, ffe3 ready)");
                    return Ok(BluetoothTransport {
                        peripheral,
                        write_char,
                        _notify_char: notify_char,
                    });
                }
            }
        }

        Err(anyhow!(
            "mBot2 not found. Make sure it's powered on and Bluetooth is enabled."
        ))
    }

    /// Initialize the CyberPi HalocodeProtocol over BLE.
    /// No boot wait needed (BLE only available after CyberPi is fully booted).
    #[cfg(feature = "bluetooth")]
    async fn init_cyberpi_protocol_ble(&mut self) -> Result<()> {
        use btleplug::api::{Peripheral as _, WriteType};
        use futures::StreamExt;

        info!("Initializing CyberPi HalocodeProtocol (BLE)...");

        // Send online mode frame
        let online = protocol::online_mode_frame();
        {
            let bt = match &self.inner {
                TransportInner::Bluetooth(bt) => bt,
                _ => return Ok(()),
            };
            debug!("TX online_mode: {:02X?}", online);
            bt.peripheral
                .write(&bt.write_char, &online, WriteType::WithoutResponse)
                .await
                .context("Failed to send online_mode over BLE")?;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Broadcast until CyberPi responds
        info!("Sending broadcast to CyberPi (BLE)...");
        for attempt in 0..10 {
            let idx = self.next_idx();
            let frame = protocol::sensor_read(protocol::read_brightness_script(), idx);

            let bt = match &self.inner {
                TransportInner::Bluetooth(bt) => bt,
                _ => return Ok(()),
            };

            debug!("TX broadcast[{}] idx={}", attempt, idx);

            bt.peripheral
                .write(&bt.write_char, &frame, WriteType::WithoutResponse)
                .await
                .context("BLE write failed")?;

            // Read notifications for up to 500ms
            let mut stream = bt.peripheral
                .notifications()
                .await
                .context("Failed to get BLE notification stream")?;

            let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
            loop {
                match tokio::time::timeout_at(deadline, stream.next()).await {
                    Ok(Some(notif)) => {
                        let responses = self.parser.feed(&notif.value);
                        if !responses.is_empty() {
                            info!("CyberPi responded via BLE! Protocol ready. (attempt {})", attempt + 1);
                            self.next_idx = 1;
                            return Ok(());
                        }
                    }
                    _ => break,
                }
            }
        }

        warn!("CyberPi did not respond to BLE broadcast after 10 attempts.");
        self.next_idx = 1;
        Ok(())
    }

    /// Send an f3 script frame over BLE and wait for matching response.
    #[cfg(feature = "bluetooth")]
    async fn query_sensor_ble(&mut self, script: &str, timeout_ms: u64) -> Option<protocol::CyberPiValue> {
        use btleplug::api::{Peripheral as _, WriteType};
        use futures::StreamExt;

        let idx = self.next_idx();
        let frame = protocol::sensor_read(script, idx);

        debug!("TX [idx={}]: {} ({} bytes)", idx, script, frame.len());

        // Write frame
        let bt = match &self.inner {
            TransportInner::Bluetooth(bt) => bt,
            _ => return None,
        };

        if let Err(e) = bt.peripheral
            .write(&bt.write_char, &frame, WriteType::WithoutResponse)
            .await
        {
            debug!("BLE TX failed: {}", e);
            return None;
        }

        // Read notifications until matching response
        let mut stream = match bt.peripheral.notifications().await {
            Ok(s) => s,
            Err(e) => {
                debug!("BLE notifications failed: {}", e);
                return None;
            }
        };

        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            match tokio::time::timeout_at(deadline, stream.next()).await {
                Ok(Some(notif)) => {
                    let responses = self.parser.feed(&notif.value);
                    for resp in responses {
                        if resp.idx == idx && resp.frame_type == protocol::TYPE_SCRIPT {
                            debug!("RX [idx={}]: {:?} ({})", resp.idx, resp.value, resp.raw_json);
                            return Some(resp.value);
                        }
                        debug!("RX [idx={} != {}]: {:?}", resp.idx, idx, resp.value);
                    }
                }
                Ok(None) => {
                    debug!("BLE notification stream ended");
                    return None;
                }
                Err(_) => {
                    debug!("RX timeout for idx={}", idx);
                    return None;
                }
            }
        }
    }

    /// Send a fire-and-forget f3 command over BLE.
    #[cfg(feature = "bluetooth")]
    async fn send_f3_command_ble(&mut self, script: &str) -> bool {
        use btleplug::api::{Peripheral as _, WriteType};

        let idx = self.next_idx();
        let frame = protocol::command(script, idx);

        debug!("TX cmd [idx={}]: {} ({} bytes)", idx, script, frame.len());

        let bt = match &self.inner {
            TransportInner::Bluetooth(bt) => bt,
            _ => return false,
        };

        match bt.peripheral
            .write(&bt.write_char, &frame, WriteType::WithoutResponse)
            .await
        {
            Ok(()) => true,
            Err(e) => {
                debug!("BLE TX failed: {}", e);
                false
            }
        }
    }

    /// Read all sensors over BLE.
    #[cfg(feature = "bluetooth")]
    async fn read_sensors_ble(&mut self) -> Result<MBotSensors> {
        let mut sensors = self.last_sensors.clone();
        let mut health = SensorHealth::default();

        const INTER_QUERY_MS: u64 = 30;
        const TIMEOUT_MS: u64 = 1000;

        if let Some(val) = self.query_sensor_ble(protocol::read_brightness_script(), TIMEOUT_MS).await {
            if let Some(v) = val.as_f32() {
                sensors.light_level = (v / 100.0).clamp(0.0, 1.0);
                health.brightness_ok = true;
            }
        }
        tokio::time::sleep(Duration::from_millis(INTER_QUERY_MS)).await;

        if let Some(val) = self.query_sensor_ble(protocol::read_loudness_script(), TIMEOUT_MS).await {
            if let Some(v) = val.as_f32() {
                sensors.sound_level = (v / 100.0).clamp(0.0, 1.0);
                health.loudness_ok = true;
            }
        }
        tokio::time::sleep(Duration::from_millis(INTER_QUERY_MS)).await;

        if let Some(val) = self.query_sensor_ble(&protocol::read_gyro_script('z'), TIMEOUT_MS).await {
            if let Some(v) = val.as_f32() {
                sensors.gyro_z = v;
                health.gyro_ok = true;
            }
        }
        tokio::time::sleep(Duration::from_millis(INTER_QUERY_MS)).await;

        let mut accel_ok = false;
        for (i, axis) in ['x', 'y', 'z'].iter().enumerate() {
            if let Some(val) = self.query_sensor_ble(&protocol::read_accel_script(*axis), TIMEOUT_MS).await {
                if let Some(v) = val.as_f32() {
                    sensors.accel[i] = v;
                    accel_ok = true;
                }
            }
            if i < 2 {
                tokio::time::sleep(Duration::from_millis(INTER_QUERY_MS)).await;
            }
        }
        health.accel_ok = accel_ok;
        tokio::time::sleep(Duration::from_millis(INTER_QUERY_MS)).await;

        if let Some(val) = self.query_sensor_ble(protocol::read_ultrasonic_script(), TIMEOUT_MS).await {
            if let Some(v) = val.as_f32() {
                sensors.ultrasonic_cm = v;
                health.ultrasonic_ok = true;
            }
        }

        sensors.timestamp_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        self.last_sensors = sensors.clone();
        self.sensor_health = health;

        Ok(sensors)
    }

    // =========================================================================
    //  Unified async interface
    // =========================================================================

    /// Query a sensor value (dispatches to serial or BLE).
    async fn query_sensor(&mut self, script: &str, timeout_ms: u64) -> Option<protocol::CyberPiValue> {
        match &self.inner {
            #[cfg(feature = "serial")]
            TransportInner::Serial(_) => self.query_sensor_serial(script, timeout_ms),
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_) => self.query_sensor_ble(script, timeout_ms).await,
            _ => None,
        }
    }

    /// Send a fire-and-forget command (dispatches to serial or BLE).
    async fn send_f3_command(&mut self, script: &str) -> bool {
        match &self.inner {
            #[cfg(feature = "serial")]
            TransportInner::Serial(_) => self.send_f3_command_serial(script),
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_) => self.send_f3_command_ble(script).await,
            _ => false,
        }
    }

    pub async fn read_sensors(&mut self) -> Result<MBotSensors> {
        match &self.inner {
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_) => self.read_sensors_ble().await,
            #[cfg(feature = "serial")]
            TransportInner::Serial(_) => self.read_sensors_serial(),
            TransportInner::Simulated => self.read_simulated(),
        }
    }

    fn read_simulated(&mut self) -> Result<MBotSensors> {
        self.sim_tick += 1;

        let wave = ((self.sim_tick as f32) * 0.02).sin();
        self.sim_distance = 50.0 + wave * 40.0;

        if self.sim_tick % 200 > 180 {
            self.sim_distance = 10.0 + (self.sim_tick % 20) as f32;
        }

        self.sim_encoder_left += 5;
        self.sim_encoder_right += 5;

        let sensors = MBotSensors {
            timestamp_us: self.sim_tick * 50_000,
            ultrasonic_cm: self.sim_distance,
            encoder_left: self.sim_encoder_left,
            encoder_right: self.sim_encoder_right,
            gyro_z: wave * 10.0,
            accel: [wave * 0.5, 0.0, 9.8],
            sound_level: 0.1 + (wave * 0.1).abs(),
            light_level: 0.5,
            quad_rgb: [[200, 200, 200]; 4],
        };

        self.sensor_health = SensorHealth {
            brightness_ok: true,
            loudness_ok: true,
            gyro_ok: true,
            accel_ok: true,
            ultrasonic_ok: true,
        };
        self.last_sensors = sensors.clone();

        Ok(sensors)
    }

    /// Run diagnostics: test each sensor individually with verbose output.
    /// Works over both serial and BLE.
    pub async fn run_diagnostics(&mut self) -> Result<SensorHealth> {
        if matches!(&self.inner, TransportInner::Simulated) {
            println!("  Diagnostics require a serial or bluetooth connection.");
            return Ok(SensorHealth::default());
        }

        let transport_name = match &self.inner {
            #[cfg(feature = "serial")]
            TransportInner::Serial(_) => "Serial",
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_) => "BLE",
            _ => "Unknown",
        };

        println!();
        println!("  ==========================================================");
        println!("    CyberPi SENSOR DIAGNOSTICS (HalocodeProtocol f3/f4)");
        println!("    Transport: {}", transport_name);
        println!("  ==========================================================");
        println!();

        let mut health = SensorHealth::default();

        let tests: Vec<(&str, &str)> = vec![
            ("Brightness (light)", protocol::read_brightness_script()),
            ("Loudness (mic)", protocol::read_loudness_script()),
            ("Battery", protocol::read_battery_script()),
            ("Roll", protocol::read_roll_script()),
            ("Pitch", protocol::read_pitch_script()),
            ("Yaw", protocol::read_yaw_script()),
        ];

        let accel_x = protocol::read_accel_script('x');
        let accel_y = protocol::read_accel_script('y');
        let accel_z = protocol::read_accel_script('z');
        let gyro_x = protocol::read_gyro_script('x');
        let gyro_y = protocol::read_gyro_script('y');
        let gyro_z = protocol::read_gyro_script('z');

        let extra_tests: Vec<(&str, &str)> = vec![
            ("Accel X", &accel_x),
            ("Accel Y", &accel_y),
            ("Accel Z", &accel_z),
            ("Gyro X", &gyro_x),
            ("Gyro Y", &gyro_y),
            ("Gyro Z", &gyro_z),
            ("Ultrasonic", protocol::read_ultrasonic_script()),
        ];

        for (label, script) in tests.iter().chain(extra_tests.iter()) {
            print!("    {:<20} -> ", label);
            match self.query_sensor(script, 1000).await {
                Some(val) => {
                    println!("OK  {:?}", val);
                    match *label {
                        "Brightness (light)" => health.brightness_ok = true,
                        "Loudness (mic)" => health.loudness_ok = true,
                        "Gyro Z" => health.gyro_ok = true,
                        "Accel Z" => health.accel_ok = true,
                        "Ultrasonic" => health.ultrasonic_ok = true,
                        _ => {}
                    }
                }
                None => println!("TIMEOUT"),
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Test actuators one at a time with pauses between each
        println!();
        println!("  ----------------------------------------------------------");
        println!("    ACTUATOR TESTS (one at a time)");
        println!("  ----------------------------------------------------------");
        println!();

        // 1. LED test - cycle through colors
        print!("    LED flash (red)      -> ");
        if self.send_f3_command("cyberpi.led.on(255, 0, 0)").await {
            println!("SENT (check robot eyes for RED)");
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.send_f3_command("cyberpi.led.on(0, 255, 0)").await;
            println!("    LED flash (green)    -> SENT");
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.send_f3_command("cyberpi.led.on(0, 0, 255)").await;
            println!("    LED flash (blue)     -> SENT");
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.send_f3_command("cyberpi.led.off()").await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        } else {
            println!("FAILED");
        }

        // 2. Display test - famous quote about machines being alive
        print!("    Display text         -> ");
        let quotes = [
            "It's alive! -Frankenstein",
            "I think therefore I am -Descartes",
            "Do androids dream? -P.K.Dick",
            "The machine is me -Ghost/Shell",
            "I learned to live -Pinocchio",
        ];
        let quote_idx = (self.sim_tick as usize + std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize) % quotes.len();
        let chosen_quote = quotes[quote_idx];
        self.send_f3_command("cyberpi.console.clear()").await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let display_cmd = protocol::display_print_script(chosen_quote);
        if self.send_f3_command(&display_cmd).await {
            println!("SENT: \"{}\"", chosen_quote);
            tokio::time::sleep(Duration::from_secs(3)).await;
        } else {
            println!("FAILED");
        }

        // 3. Microphone responsiveness test
        println!();
        println!("  ----------------------------------------------------------");
        println!("    MICROPHONE TEST (make some noise!)");
        println!("  ----------------------------------------------------------");
        println!();
        // Show instruction on the CyberPi display so the user knows what to do
        self.send_f3_command("cyberpi.console.clear()").await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.send_f3_command(&protocol::display_print_script("MIC TEST")).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.send_f3_command(&protocol::display_print_script("Clap or speak!")).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.send_f3_command(&protocol::display_print_script("Listening...")).await;
        println!("    Reading mic levels for 5 seconds... clap, speak, or tap the robot!");
        println!();
        let mut max_loudness: f64 = 0.0;
        let mut min_loudness: f64 = 100.0;
        for i in 0..10 {
            match self.query_sensor(protocol::read_loudness_script(), 500).await {
                Some(val) => {
                    let level = val.as_f64().unwrap_or(0.0);
                    let bar_len = (level / 2.0).round() as usize; // scale to ~50 chars
                    let bar_len = bar_len.min(50);
                    // Show on terminal
                    println!(
                        "    [{:>2}/10] loudness={:<6.1}  [{}{}]",
                        i + 1,
                        level,
                        "#".repeat(bar_len),
                        "-".repeat(50 - bar_len),
                    );
                    // Show live level on CyberPi display so user sees what mic hears
                    let display_bar_len = (level / 10.0).round() as usize;
                    let display_bar_len = display_bar_len.min(10);
                    let display_msg = format!(
                        "Loud: {} {}",
                        "#".repeat(display_bar_len),
                        level as i64,
                    );
                    self.send_f3_command(&protocol::display_print_script(&display_msg)).await;
                    if level > max_loudness { max_loudness = level; }
                    if level < min_loudness { min_loudness = level; }
                }
                None => {
                    println!("    [{:>2}/10] TIMEOUT", i + 1);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        println!();
        println!("    Mic range: min={:.1} max={:.1} delta={:.1}",
            min_loudness, max_loudness, max_loudness - min_loudness);
        if max_loudness - min_loudness > 5.0 {
            println!("    Mic: RESPONSIVE (detected sound variation)");
            self.send_f3_command("cyberpi.console.clear()").await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.send_f3_command(&protocol::display_print_script("Mic: OK!")).await;
            self.send_f3_command(&protocol::display_print_script(
                &format!("Peak: {}", max_loudness as i64)
            )).await;
            self.send_f3_command(&protocol::display_print_script("I heard you!")).await;
        } else {
            println!("    Mic: LOW VARIATION (quiet environment or mic issue)");
            self.send_f3_command("cyberpi.console.clear()").await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.send_f3_command(&protocol::display_print_script("Mic: quiet")).await;
            self.send_f3_command(&protocol::display_print_script("Try clapping!")).await;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!();

        // 4. Speaker test - built-in sounds + musical scale
        println!("  ----------------------------------------------------------");
        println!("    SPEAKER TEST");
        println!("  ----------------------------------------------------------");
        println!();
        // CyberPi has built-in sound effects: hello, yeah, wow, laugh, etc.
        self.send_f3_command("cyberpi.console.clear()").await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.send_f3_command(&protocol::display_print_script("SPEAKER TEST")).await;

        // Try built-in voice sounds first
        println!("    Built-in sounds:");
        let sounds = ["hello", "yeah", "wow", "laugh"];
        for sound in &sounds {
            print!("    Playing '{}'... ", sound);
            self.send_f3_command(&format!("cyberpi.audio.play('{}')", sound)).await;
            self.send_f3_command(&protocol::display_print_script(sound)).await;
            println!("SENT");
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
        println!();

        // Musical scale
        println!("    Playing ascending scale...");
        // C major scale: C5, D5, E5, F5, G5
        let notes = [(523, "C5"), (587, "D5"), (659, "E5"), (698, "F5"), (784, "G5")];
        for (freq, name) in &notes {
            self.send_f3_command(&format!("cyberpi.audio.play_tone({}, 0.4)", freq)).await;
            println!("    {} ({}Hz)", name, freq);
            tokio::time::sleep(Duration::from_millis(550)).await;
        }
        // Descending flourish
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.send_f3_command("cyberpi.audio.play_tone(784, 0.2)").await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        self.send_f3_command("cyberpi.audio.play_tone(659, 0.2)").await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        self.send_f3_command("cyberpi.audio.play_tone(523, 0.6)").await;
        tokio::time::sleep(Duration::from_millis(800)).await;
        println!("    (C major scale + flourish)");
        println!();

        // 5. Motor test: forward, then one circle, then stop
        println!("  ----------------------------------------------------------");
        println!("    MOTOR TEST");
        println!("  ----------------------------------------------------------");
        println!();
        println!("    Motor test           -> forward 1 second");
        self.send_f3_command("mbot2.drive_speed(30, 30)").await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.send_f3_command("mbot2.drive_speed(0, 0)").await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        println!("    Motor test           -> one circle (spin right)");
        self.send_f3_command("mbot2.drive_speed(30, -30)").await;
        tokio::time::sleep(Duration::from_millis(2000)).await;
        self.send_f3_command("mbot2.drive_speed(0, 0)").await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("                            motors stopped");

        // Ensure motors are stopped at end of diagnostics
        self.send_f3_command("mbot2.drive_speed(0, 0)").await;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let ok = health.ok_count();
        let total = health.total();
        println!();
        println!("  ==========================================================");
        println!("    {}/{} core sensors responding", ok, total);
        if ok < total {
            println!("    Failed: {}", health.failed_names().join(", "));
            println!("    Note: Ultrasonic requires mBot2 shield + sensor module");
        }
        println!("  ==========================================================");
        println!();

        self.sensor_health = health.clone();
        Ok(health)
    }

    // =================================================================
    // R2-D2 Robot Voice (VCONV-001, VCONV-008)
    // =================================================================

    /// Speak text as R2-D2 tone patterns through CyberPi's speaker.
    /// Simultaneously displays text on screen and pulses LEDs.
    ///
    /// Two modes:
    /// - `local=true`: Uploads a Python loop to CyberPi that plays locally (lower latency)
    /// - `local=false`: Sends individual play_tone() commands from laptop (more control)
    pub async fn robot_speak(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        // Show text on display first (VCONV-008: sync text with tones)
        self.send_f3_command("cyberpi.console.clear()").await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Split text into lines that fit the CyberPi display (~16 chars)
        for chunk in text.as_bytes().chunks(16) {
            let line = String::from_utf8_lossy(chunk);
            self.send_f3_command(&protocol::display_print_script(&line)).await;
            tokio::time::sleep(Duration::from_millis(30)).await;
        }

        // Pulse LEDs green while speaking
        self.send_f3_command("cyberpi.led.on(0, 80, 40)").await;

        // Use local CyberPi exec for lower latency R2-D2 voice
        // The exec script runs a for loop on CyberPi itself, playing tones for each char
        let exec_script = protocol::r2d2_exec_script(text, 200, 6, 80);
        self.send_f3_command(&exec_script).await;

        // Wait for approximate playback duration (CyberPi runs tones locally)
        // Each char takes ~80ms tone + ~40ms gap = ~120ms
        let approx_duration_ms = text.len() as u64 * 120;
        tokio::time::sleep(Duration::from_millis(approx_duration_ms.min(15000))).await;

        // Small extra delay to let CyberPi finish processing before next BLE command
        tokio::time::sleep(Duration::from_millis(200)).await;

        // LEDs off after speaking
        self.send_f3_command("cyberpi.led.off()").await;

        Ok(())
    }

    /// Speak with individual tone commands from laptop (more control, higher latency).
    /// Uses play_tone() for each character, with LED pulse and display text.
    pub async fn robot_speak_detailed(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        // Show text on display (VCONV-008: sync)
        self.send_f3_command("cyberpi.console.clear()").await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        for chunk in text.as_bytes().chunks(16) {
            let line = String::from_utf8_lossy(chunk);
            self.send_f3_command(&protocol::display_print_script(&line)).await;
            tokio::time::sleep(Duration::from_millis(30)).await;
        }

        // Pulse LEDs
        self.send_f3_command("cyberpi.led.on(0, 80, 40)").await;

        // Play tones individually from laptop
        let tones = protocol::text_to_r2d2_tones(text, 200, 6, 80);
        for (freq, dur_ms) in &tones {
            if *freq == 0 {
                // Silence/pause
                tokio::time::sleep(Duration::from_millis(*dur_ms as u64)).await;
            } else {
                let script = format!("cyberpi.audio.play_tone({}, {:.3})", freq, *dur_ms as f32 / 1000.0);
                self.send_f3_command(&script).await;
                // Wait for tone duration + small gap
                tokio::time::sleep(Duration::from_millis(*dur_ms as u64 + 30)).await;
            }
        }

        // LEDs off
        self.send_f3_command("cyberpi.led.off()").await;

        Ok(())
    }

    pub async fn send_command(&mut self, cmd: &MotorCommand) -> Result<()> {
        match &self.inner {
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_) => {
                self.send_f3_command(&protocol::motor_script(cmd.left, cmd.right)).await;

                if cmd.led_color != [0, 0, 0] {
                    self.send_f3_command(&protocol::led_script(
                        cmd.led_color[0], cmd.led_color[1], cmd.led_color[2],
                    )).await;
                }

                if cmd.pen_angle != 45 {
                    self.send_f3_command(&protocol::servo_script(1, cmd.pen_angle)).await;
                }

                if cmd.buzzer_hz > 0 {
                    self.send_f3_command(&protocol::buzzer_script(cmd.buzzer_hz, 100)).await;
                }

                Ok(())
            }
            #[cfg(feature = "serial")]
            TransportInner::Serial(_) => {
                self.send_f3_command(&protocol::motor_script(cmd.left, cmd.right)).await;

                if cmd.led_color != [0, 0, 0] {
                    self.send_f3_command(&protocol::led_script(
                        cmd.led_color[0], cmd.led_color[1], cmd.led_color[2],
                    )).await;
                }

                if cmd.pen_angle != 45 {
                    self.send_f3_command(&protocol::servo_script(1, cmd.pen_angle)).await;
                }

                if cmd.buzzer_hz > 0 {
                    self.send_f3_command(&protocol::buzzer_script(cmd.buzzer_hz, 100)).await;
                }

                Ok(())
            }
            TransportInner::Simulated => {
                debug!(
                    "SIM Command: L={} R={} Mode={:?}",
                    cmd.left,
                    cmd.right,
                    if cmd.left < 0 && cmd.right < 0 {
                        "REVERSE"
                    } else if cmd.left > cmd.right {
                        "TURN_RIGHT"
                    } else if cmd.right > cmd.left {
                        "TURN_LEFT"
                    } else {
                        "FORWARD"
                    }
                );
                Ok(())
            }
        }
    }
}
