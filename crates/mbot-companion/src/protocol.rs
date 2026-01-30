//! Makeblock mBot2 protocol implementation
//!
//! Based on Makeblock's protocol documentation.
//! Reference: https://github.com/Makeblock-official/Makeblock-Libraries

/// Protocol header
const HEADER: [u8; 2] = [0xff, 0x55];

/// Device IDs
#[allow(dead_code)]
mod device {
    pub const ULTRASONIC: u8 = 0x01;
    pub const LIGHT_SENSOR: u8 = 0x03;
    pub const SOUND_SENSOR: u8 = 0x07;
    pub const GYRO: u8 = 0x06;
    pub const DC_MOTOR: u8 = 0x0a;
    pub const ENCODER_MOTOR: u8 = 0x3d;
    pub const SERVO: u8 = 0x0b;
    pub const RGBLED: u8 = 0x08;
    pub const BUZZER: u8 = 0x22;
    pub const QUAD_RGB: u8 = 0x41;
}

/// Action codes
#[allow(dead_code)]
mod action {
    pub const GET: u8 = 0x01;
    pub const RUN: u8 = 0x02;
}

/// Build ultrasonic sensor read command
pub fn read_ultrasonic_cmd() -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        0x04,              // Length
        0x00,              // Index (for response matching)
        action::GET,       // Action: GET
        device::ULTRASONIC,// Device: Ultrasonic
        0x03,              // Port 3 (default mBot2 position)
    ]
}

/// Parse ultrasonic response
pub fn parse_ultrasonic_response(data: &[u8]) -> Option<f32> {
    // Response format: [0xff, 0x55, index, type, data...]
    if data.len() < 5 {
        return None;
    }

    if data[0] != 0xff || data[1] != 0x55 {
        return None;
    }

    // Type 2 = float response
    if data[3] == 0x02 && data.len() >= 8 {
        let bytes = [data[4], data[5], data[6], data[7]];
        Some(f32::from_le_bytes(bytes))
    } else {
        None
    }
}

/// Build motor command
pub fn motor_cmd(left: i8, right: i8) -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        0x08,              // Length
        0x00,              // Index
        action::RUN,       // Action: RUN
        device::DC_MOTOR,  // Device: DC Motor
        0x00,              // Port: Both motors
        left as u8,        // Left motor speed
        right as u8,       // Right motor speed
    ]
}

/// Build encoder motor command (more precise)
pub fn encoder_motor_cmd(port: u8, speed: i16, position: Option<i32>) -> Vec<u8> {
    let mut cmd = vec![
        HEADER[0],
        HEADER[1],
        0x00,                   // Length (filled later)
        0x00,                   // Index
        action::RUN,            // Action: RUN
        device::ENCODER_MOTOR,  // Device: Encoder Motor
        port,                   // Port
        0x02,                   // Slot
    ];

    // Add speed (2 bytes)
    cmd.extend_from_slice(&speed.to_le_bytes());

    // Add position if specified
    if let Some(pos) = position {
        cmd.extend_from_slice(&pos.to_le_bytes());
    }

    // Update length
    cmd[2] = (cmd.len() - 3) as u8;

    cmd
}

/// Build servo command
pub fn servo_cmd(port: u8, angle: u8) -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        0x05,           // Length
        0x00,           // Index
        action::RUN,    // Action: RUN
        device::SERVO,  // Device: Servo
        port,           // Port
        angle,          // Angle (0-180)
    ]
}

/// Build RGB LED command
pub fn led_cmd(rgb: [u8; 3]) -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        0x08,           // Length
        0x00,           // Index
        action::RUN,    // Action: RUN
        device::RGBLED, // Device: RGB LED
        0x07,           // Port: Onboard
        0x02,           // Slot: All LEDs
        0x00,           // LED index (0 = all)
        rgb[0],         // Red
        rgb[1],         // Green
        rgb[2],         // Blue
    ]
}

/// Build buzzer command
pub fn buzzer_cmd(frequency: u16, duration_ms: u16) -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        0x08,            // Length
        0x00,            // Index
        action::RUN,     // Action: RUN
        device::BUZZER,  // Device: Buzzer
        (frequency & 0xff) as u8,
        (frequency >> 8) as u8,
        (duration_ms & 0xff) as u8,
        (duration_ms >> 8) as u8,
    ]
}

/// Build gyro read command
pub fn read_gyro_cmd(axis: u8) -> Vec<u8> {
    // axis: 1=X, 2=Y, 3=Z
    vec![
        HEADER[0],
        HEADER[1],
        0x05,          // Length
        0x00,          // Index
        action::GET,   // Action: GET
        device::GYRO,  // Device: Gyro
        0x00,          // Port (onboard)
        axis,          // Axis
    ]
}

/// Build quad RGB sensor read command
pub fn read_quad_rgb_cmd() -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        0x04,             // Length
        0x00,             // Index
        action::GET,      // Action: GET
        device::QUAD_RGB, // Device: Quad RGB
        0x01,             // Port 1
    ]
}

/// Parse quad RGB sensor response
/// Returns (r, g, b) values (0-255 per channel)
pub fn parse_quad_rgb_response(data: &[u8]) -> Option<(u8, u8, u8)> {
    // Response format: [0xff, 0x55, index, type, r, g, b, ...]
    if data.len() < 7 {
        return None;
    }

    if data[0] != 0xff || data[1] != 0x55 {
        return None;
    }

    // Extract RGB values (bytes 4, 5, 6)
    Some((data[4], data[5], data[6]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_cmd() {
        let cmd = motor_cmd(50, 50);
        assert_eq!(cmd[0], 0xff);
        assert_eq!(cmd[1], 0x55);
        assert_eq!(cmd[5], device::DC_MOTOR);
        assert_eq!(cmd[7], 50);
        assert_eq!(cmd[8], 50);
    }

    #[test]
    fn test_led_cmd() {
        let cmd = led_cmd([255, 0, 0]); // Red
        assert_eq!(cmd[0], 0xff);
        assert_eq!(cmd[1], 0x55);
        assert_eq!(cmd[5], device::RGBLED);
        assert_eq!(cmd[9], 255);
        assert_eq!(cmd[10], 0);
        assert_eq!(cmd[11], 0);
    }

    #[test]
    fn test_servo_cmd() {
        let cmd = servo_cmd(1, 90);
        assert_eq!(cmd[0], 0xff);
        assert_eq!(cmd[1], 0x55);
        assert_eq!(cmd[5], device::SERVO);
        assert_eq!(cmd[6], 1);
        assert_eq!(cmd[7], 90);
    }

    #[test]
    fn test_servo_cmd_pen_up() {
        // ART-001: Pen up position at 45°
        let cmd = servo_cmd(1, 45);
        assert_eq!(cmd.len(), 8);
        assert_eq!(cmd[0], 0xff);
        assert_eq!(cmd[1], 0x55);
        assert_eq!(cmd[2], 0x05);        // Length
        assert_eq!(cmd[3], 0x00);        // Index
        assert_eq!(cmd[4], action::RUN); // Action
        assert_eq!(cmd[5], device::SERVO); // Device
        assert_eq!(cmd[6], 1);           // Port
        assert_eq!(cmd[7], 45);          // Angle (pen up)
    }

    #[test]
    fn test_servo_cmd_pen_down() {
        // ART-001: Pen down position at 90°
        let cmd = servo_cmd(1, 90);
        assert_eq!(cmd.len(), 8);
        assert_eq!(cmd[7], 90);          // Angle (pen down)
    }

    #[test]
    fn test_servo_cmd_all_valid_angles() {
        // ART-001: Verify servo accuracy - test range of angles
        for angle in (0..=180).step_by(5) {
            let cmd = servo_cmd(1, angle);
            assert_eq!(cmd.len(), 8);
            assert_eq!(cmd[7], angle);
        }
    }

    #[test]
    fn test_servo_cmd_different_ports() {
        // Test servo on different ports
        for port in 1..=4 {
            let cmd = servo_cmd(port, 90);
            assert_eq!(cmd[6], port);
            assert_eq!(cmd[7], 90);
        }
    }

    #[test]
    fn test_servo_cmd_header() {
        // Verify protocol header is always correct
        let cmd = servo_cmd(1, 90);
        assert_eq!(cmd[0], 0xff);
        assert_eq!(cmd[1], 0x55);
    }

    #[test]
    fn test_servo_cmd_length() {
        // Verify command length field (byte 2)
        let cmd = servo_cmd(1, 90);
        assert_eq!(cmd[2], 0x05); // Length is always 5 bytes
    }

    #[test]
    fn test_parse_ultrasonic() {
        // Simulate response: 25.5 cm
        let distance: f32 = 25.5;
        let bytes = distance.to_le_bytes();
        let response = vec![0xff, 0x55, 0x00, 0x02, bytes[0], bytes[1], bytes[2], bytes[3]];

        let parsed = parse_ultrasonic_response(&response);
        assert!(parsed.is_some());
        assert!((parsed.unwrap() - 25.5).abs() < 0.01);
    }

    #[test]
    fn test_read_quad_rgb_cmd() {
        let cmd = read_quad_rgb_cmd();
        assert_eq!(cmd[0], 0xff);
        assert_eq!(cmd[1], 0x55);
        assert_eq!(cmd[2], 0x04); // Length
        assert_eq!(cmd[4], action::GET);
        assert_eq!(cmd[5], device::QUAD_RGB);
        assert_eq!(cmd[6], 0x01); // Port 1
    }

    #[test]
    fn test_parse_quad_rgb_response() {
        // Simulate RGB response: R=200, G=100, B=50
        let response = vec![0xff, 0x55, 0x00, 0x03, 200, 100, 50];

        let parsed = parse_quad_rgb_response(&response);
        assert!(parsed.is_some());
        let (r, g, b) = parsed.unwrap();
        assert_eq!(r, 200);
        assert_eq!(g, 100);
        assert_eq!(b, 50);
    }

    #[test]
    fn test_parse_quad_rgb_invalid() {
        // Too short
        let short = vec![0xff, 0x55, 0x00];
        assert!(parse_quad_rgb_response(&short).is_none());

        // Invalid header
        let bad_header = vec![0xaa, 0xbb, 0x00, 0x03, 200, 100, 50];
        assert!(parse_quad_rgb_response(&bad_header).is_none());
    }
}
