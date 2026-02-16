//! CyberPi HalocodeProtocol implementation
//!
//! CyberPi uses f3/f4 framing (shared with Halocode). The protocol sends Python
//! expressions inside binary frames; CyberPi evaluates them and returns JSON
//! results. This is how mBlock "Live Mode" works.
//!
//! Frame format:
//!   f3 [hdr_check] [datalen_lo] [datalen_hi] [type] [mode] [idx_lo] [idx_hi] [data...] [checksum] f4
//!
//! Where:
//!   hdr_check = ((datalen_hi) + (datalen_lo) + 0xf3) & 0xff
//!   datalen   = len(data) + 4  (type + mode + idx_lo + idx_hi)
//!   checksum  = (type + mode + idx_lo + idx_hi + sum(data)) & 0xff
//!
//! For TYPE_SCRIPT, data = [script_len_lo, script_len_hi, script_bytes...]
//! Response data contains JSON like {"ret": 42}
//!
//! Reference: makeblock pip package v0.1.8, protocols/PackData.py

/// Protocol constants
pub const HEADER: u8 = 0xf3;
pub const FOOTER: u8 = 0xf4;

pub const TYPE_SCRIPT: u8 = 0x28;
pub const TYPE_SUBSCRIBE: u8 = 0x29;
pub const TYPE_ONLINE: u8 = 0x0d;

pub const MODE_RUN_WITHOUT_RESPONSE: u8 = 0x00;
pub const MODE_RUN_WITH_RESPONSE: u8 = 0x01;

/// Build a script frame that sends a Python expression to CyberPi for evaluation.
/// Returns the response keyed by `idx`.
pub fn script_frame(script: &str, idx: u16, mode: u8) -> Vec<u8> {
    let script_bytes = script.as_bytes();
    let script_len = script_bytes.len();

    // data = [script_len_lo, script_len_hi, script_bytes...]
    let mut data = Vec::with_capacity(2 + script_len);
    data.push((script_len & 0xff) as u8);
    data.push(((script_len >> 8) & 0xff) as u8);
    data.extend_from_slice(script_bytes);

    let typ = TYPE_SCRIPT;
    let idx_lo = (idx & 0xff) as u8;
    let idx_hi = ((idx >> 8) & 0xff) as u8;

    // datalen includes type + mode + idx_lo + idx_hi + data
    let datalen = data.len() + 4;

    let mut buf = Vec::with_capacity(datalen + 6);
    buf.push(HEADER);

    // Header check byte
    let hdr_check = (((datalen >> 8) & 0xff) + (datalen & 0xff) + 0xf3) & 0xff;
    buf.push(hdr_check as u8);

    buf.push((datalen & 0xff) as u8);
    buf.push(((datalen >> 8) & 0xff) as u8);

    buf.push(typ);
    buf.push(mode);
    buf.push(idx_lo);
    buf.push(idx_hi);

    buf.extend_from_slice(&data);

    // Checksum
    let mut cksum: u32 = typ as u32 + mode as u32 + idx_lo as u32 + idx_hi as u32;
    for &b in &data {
        cksum += b as u32;
    }
    buf.push((cksum & 0xff) as u8);

    buf.push(FOOTER);
    buf
}

/// Build the goto_online_mode frame.
/// This must be sent first to put CyberPi into "Live Mode".
pub fn online_mode_frame() -> Vec<u8> {
    vec![0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x01, 0x0e, 0xf4]
}

/// Build a sensor read frame. Shorthand for script_frame with RUN_WITH_RESPONSE.
pub fn sensor_read(script: &str, idx: u16) -> Vec<u8> {
    script_frame(script, idx, MODE_RUN_WITH_RESPONSE)
}

/// Build a fire-and-forget command frame (no response expected).
pub fn command(script: &str, idx: u16) -> Vec<u8> {
    script_frame(script, idx, MODE_RUN_WITHOUT_RESPONSE)
}

// ---------------------------------------------------------------------------
// Sensor read scripts (Python expressions evaluated on CyberPi)
// ---------------------------------------------------------------------------

/// Brightness sensor (0-100)
pub fn read_brightness_script() -> &'static str {
    "cyberpi.get_bri()"
}

/// Loudness / microphone level (0-100)
pub fn read_loudness_script() -> &'static str {
    "cyberpi.get_loudness('maximum')"
}

/// Battery percentage (0-100)
pub fn read_battery_script() -> &'static str {
    "cyberpi.get_battery()"
}

/// Roll angle (degrees)
pub fn read_roll_script() -> &'static str {
    "cyberpi.get_roll()"
}

/// Pitch angle (degrees)
pub fn read_pitch_script() -> &'static str {
    "cyberpi.get_pitch()"
}

/// Yaw angle (degrees)
pub fn read_yaw_script() -> &'static str {
    "cyberpi.get_yaw()"
}

/// Accelerometer axis ('x', 'y', or 'z') - returns m/s^2
pub fn read_accel_script(axis: char) -> String {
    format!("cyberpi.get_acc('{}')", axis)
}

/// Gyroscope axis ('x', 'y', or 'z') - returns deg/s
pub fn read_gyro_script(axis: char) -> String {
    format!("cyberpi.get_gyro('{}')", axis)
}

/// Ultrasonic distance on mBot2 shield (port 1 or 2) - returns cm
/// Note: requires mbot2 module, not cyberpi module
pub fn read_ultrasonic_script() -> &'static str {
    "mbot2.ultrasonic2.get(1)"
}

// ---------------------------------------------------------------------------
// Actuator command scripts
// ---------------------------------------------------------------------------

/// Drive both motors (speed range: -100 to 100)
///
/// Always uses high-level mbot2 commands (forward/backward/turn_right/turn_left)
/// which handle motor calibration and direction mapping internally.
/// drive_speed() has inverted semantics (same sign = spin) so we avoid it.
pub fn motor_script(left: i8, right: i8) -> String {
    if left == 0 && right == 0 {
        "mbot2.forward(0)".into()
    } else if left == right && left > 0 {
        format!("mbot2.forward({})", left)
    } else if left == right && left < 0 {
        format!("mbot2.backward({})", -left)
    } else if left > 0 && right < 0 {
        // Spin right (left forward, right backward)
        let speed = (left.abs().max(right.abs())) as i16;
        format!("mbot2.turn_right({})", speed)
    } else if left < 0 && right > 0 {
        // Spin left (left backward, right forward)
        let speed = (left.abs().max(right.abs())) as i16;
        format!("mbot2.turn_left({})", speed)
    } else {
        // Asymmetric forward/arc — use forward at average speed
        let avg = ((left as i16 + right as i16) / 2).abs();
        if left > right {
            // Arc right
            format!("mbot2.turn_right({})", avg.max(20))
        } else {
            // Arc left
            format!("mbot2.turn_left({})", avg.max(20))
        }
    }
}

/// Set LED color on CyberPi (all LEDs, RGB)
pub fn led_script(r: u8, g: u8, b: u8) -> String {
    format!("cyberpi.led.on({}, {}, {})", r, g, b)
}

/// Play buzzer tone
pub fn buzzer_script(freq: u16, duration_ms: u16) -> String {
    let duration_s = duration_ms as f32 / 1000.0;
    format!("cyberpi.audio.play_tone({}, {:.2})", freq, duration_s)
}

/// Set servo angle (port, angle 0-180)
pub fn servo_script(port: u8, angle: u8) -> String {
    format!("mbot2.servo_set({}, {})", port, angle)
}

/// Print to CyberPi console display
pub fn display_print_script(text: &str) -> String {
    format!("cyberpi.console.println('{}')", text.replace('\'', "\\'"))
}

/// Generate R2-D2 tone sequence from text.
/// Maps each character to a frequency: base_freq + (char_code * multiplier).
/// Spaces and punctuation get special treatment for natural rhythm.
pub fn text_to_r2d2_tones(text: &str, base_freq: u16, multiplier: u16, tone_ms: u16) -> Vec<(u16, u16)> {
    let mut tones = Vec::new();
    for ch in text.chars() {
        if ch == ' ' {
            // Silence gap for word boundaries (frequency 0 = pause)
            tones.push((0, tone_ms / 2));
        } else if ch == '!' || ch == '?' {
            // Excited punctuation: high chirp
            tones.push((base_freq + 600, tone_ms + 40));
        } else if ch == '.' || ch == ',' {
            // Pause for sentence breaks
            tones.push((0, tone_ms));
        } else {
            let code = (ch as u16).min(127);
            let freq = base_freq + (code.wrapping_mul(multiplier) % 800);
            tones.push((freq, tone_ms));
        }
    }
    tones
}

/// Build a CyberPi exec() script that defines a local say() function.
/// This runs entirely on the CyberPi for lower latency.
pub fn r2d2_exec_script(text: &str, base_freq: u16, multiplier: u16, tone_ms: u16) -> String {
    // Build a Python one-liner that plays tones for each character
    // exec("for c in 'text': cyberpi.audio.play_tone(200+ord(c)*6%800, 0.08) if c!=' ' else __import__('time').sleep(0.04)")
    let escaped = text.replace('\'', "\\'").replace('\\', "\\\\");
    let dur_s = tone_ms as f32 / 1000.0;
    let pause_s = (tone_ms as f32 / 2.0) / 1000.0;
    format!(
        "exec(\"for c in '{}': cyberpi.audio.play_tone({}+ord(c)*{}%800, {:.3}) if c not in ' .,!?' else __import__('time').sleep({:.3})\")",
        escaped, base_freq, multiplier, dur_s, pause_s
    )
}

// ---------------------------------------------------------------------------
// Response parser
// ---------------------------------------------------------------------------

/// Parsed response from a CyberPi f3 frame.
#[derive(Debug, Clone)]
pub enum CyberPiValue {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
    None,
}

impl CyberPiValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CyberPiValue::Float(f) => Some(*f),
            CyberPiValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|f| f as f32)
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            CyberPiValue::Int(i) => Some(*i),
            CyberPiValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            CyberPiValue::Text(s) => Some(s),
            _ => None,
        }
    }
}

/// A fully parsed f3 response frame.
#[derive(Debug, Clone)]
pub struct F3Response {
    pub idx: u16,
    pub frame_type: u8,
    pub value: CyberPiValue,
    /// Raw JSON string from the response (e.g. `{"ret": 42}`)
    pub raw_json: String,
}

/// Streaming f3/f4 frame parser. Feed bytes one at a time or in chunks.
/// Mirrors HalocodeProtocol.on_parse() from the makeblock library.
pub struct F3Parser {
    buffer: Vec<u8>,
    is_receiving: bool,
    datalen: usize,
    /// Whether we've received at least one valid frame.
    pub ready: bool,
}

impl F3Parser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
            is_receiving: false,
            datalen: 0,
            ready: false,
        }
    }

    /// Feed a chunk of bytes. Returns any complete parsed responses.
    pub fn feed(&mut self, data: &[u8]) -> Vec<F3Response> {
        let mut responses = Vec::new();
        for &byte in data {
            if let Some(resp) = self.feed_byte(byte) {
                responses.push(resp);
            }
        }
        responses
    }

    /// Feed a single byte. Returns a parsed response if a complete frame was received.
    pub fn feed_byte(&mut self, byte: u8) -> Option<F3Response> {
        self.buffer.push(byte);

        // Detect header: check if last 4 bytes form a valid f3 header
        if self.buffer.len() > 3 {
            let len = self.buffer.len();
            let b3 = self.buffer[len - 4]; // should be 0xf3
            let b2 = self.buffer[len - 3]; // header check
            let b1 = self.buffer[len - 2]; // datalen_lo
            let b0 = self.buffer[len - 1]; // datalen_hi

            if b3 == HEADER && ((b0 as u16 + b1 as u16 + b3 as u16) & 0xff) as u8 == b2 {
                // Valid header found - reset buffer to just these 4 bytes
                self.buffer = vec![HEADER, b2, b1, b0];
                self.datalen = b1 as usize + ((b0 as usize) << 8);
                self.is_receiving = true;
            }
        }

        if self.is_receiving {
            // Complete frame check: buffer should be datalen + 6 (4 header + checksum + footer)
            let expected_len = self.datalen + 6;
            if self.buffer.len() == expected_len && self.buffer[0] == HEADER {
                self.ready = true;
                let frame = std::mem::take(&mut self.buffer);
                self.is_receiving = false;
                self.datalen = 0;
                return self.parse_frame(&frame);
            }
            // Safety: if buffer grows too large, something went wrong
            if self.buffer.len() > 4096 {
                self.buffer.clear();
                self.is_receiving = false;
                self.datalen = 0;
            }
        } else {
            // Not receiving - keep buffer from growing unbounded
            // Only keep last 4 bytes for header detection
            if self.buffer.len() > 64 {
                let start = self.buffer.len() - 4;
                self.buffer = self.buffer[start..].to_vec();
            }
        }

        None
    }

    /// Parse a complete f3 frame into a response.
    fn parse_frame(&self, frame: &[u8]) -> Option<F3Response> {
        // Frame layout:
        // 0: f3 (header)
        // 1: hdr_check
        // 2: datalen_lo
        // 3: datalen_hi
        // 4: type
        // 5: mode (for TYPE_SCRIPT) or other
        // 6: idx_lo (for TYPE_SCRIPT)
        // 7: idx_hi (for TYPE_SCRIPT)
        // 8..end-2: data
        // end-2: checksum
        // end-1: f4 (footer)

        if frame.len() < 8 {
            return None;
        }

        let frame_type = frame[4];

        match frame_type {
            TYPE_SCRIPT => self.parse_script_response(frame),
            TYPE_ONLINE => {
                // Online mode acknowledgment
                Some(F3Response {
                    idx: 0,
                    frame_type: TYPE_ONLINE,
                    value: CyberPiValue::None,
                    raw_json: String::new(),
                })
            }
            _ => None, // Unknown frame types are ignored
        }
    }

    /// Parse a TYPE_SCRIPT (0x28) response.
    /// The payload after idx contains: [data_len_lo, data_len_hi, json_bytes...]
    /// Where json_bytes is something like `{"ret": 42}`
    fn parse_script_response(&self, frame: &[u8]) -> Option<F3Response> {
        if frame.len() < 10 {
            return None;
        }

        let idx = frame[6] as u16 + ((frame[7] as u16) << 8);

        // Data payload is between idx_hi+1 and checksum
        // frame[8..frame.len()-2] = full data including script_len prefix
        let data = &frame[8..frame.len() - 2];

        // data[0..2] = script_len (response data length), data[2..] = actual JSON
        // But from observation, data[0] = len_lo, data[1] = len_hi, data[2] = 0x00 (null),
        // then the JSON starts. Let me look at the raw frames again:
        //
        // f3 03 10 00 28 01 01 00 0a 00 7b 22 72 65 74 22 3a 37 35 7d 61 f4
        // idx=8: 0a 00 = script data, then 7b 22 = {"
        //
        // So data = [0x0a, 0x00, 0x7b, 0x22, ...] where 0x0a = 10 = length of '{"ret":75}'
        // The JSON starts at data[2], not data[3] as the makeblock lib does for its own
        // callback... wait, the makeblock lib uses pack.data[3:] because its data array
        // starts differently. Let me re-examine.
        //
        // In HalocodePackData parsing (line 186):
        //   self._data = buf[7:end-2]  -- includes idx_hi and everything after
        //
        // Then in common_request_response_cb (modules.py line 115):
        //   ret = eval(str(bytes(pack.data[3:len(pack.data)]), 'utf-8'))
        //
        // pack.data = buf[7:end-2], so data[3:] skips: buf[7]=idx_hi, buf[8]=data_lo, buf[9]=data_hi
        // which means the JSON starts at buf[10] = our frame[10] = data[2]
        //
        // So for our `data` (frame[8..end-2]), the JSON starts at data[2].

        if data.len() < 3 {
            return None;
        }

        let json_bytes = &data[2..];
        let raw_json = String::from_utf8_lossy(json_bytes).to_string();

        // Parse the JSON value - it's Python repr: {"ret": 42} or {"ret": "text"}
        let value = parse_ret_value(&raw_json);

        Some(F3Response {
            idx,
            frame_type: TYPE_SCRIPT,
            value,
            raw_json,
        })
    }
}

/// Parse a `{"ret": ...}` JSON string into a CyberPiValue.
/// CyberPi returns Python repr which is close to JSON but uses single quotes
/// for strings and True/False/None instead of true/false/null.
fn parse_ret_value(s: &str) -> CyberPiValue {
    // Try serde_json first (handles standard JSON)
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
        if let Some(ret) = v.get("ret") {
            return json_to_value(ret);
        }
    }

    // Fallback: manual extraction for Python-style repr
    // Look for "ret": or 'ret': followed by the value
    let ret_patterns = ["\"ret\":", "'ret':"];
    for pattern in ret_patterns {
        if let Some(pos) = s.find(pattern) {
            let after = s[pos + pattern.len()..].trim();
            // Strip trailing }
            let val_str = after.trim_end_matches('}').trim();
            return parse_python_value(val_str);
        }
    }

    CyberPiValue::None
}

fn json_to_value(v: &serde_json::Value) -> CyberPiValue {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CyberPiValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                CyberPiValue::Float(f)
            } else {
                CyberPiValue::None
            }
        }
        serde_json::Value::String(s) => CyberPiValue::Text(s.clone()),
        serde_json::Value::Bool(b) => CyberPiValue::Bool(*b),
        serde_json::Value::Null => CyberPiValue::None,
        _ => CyberPiValue::Text(v.to_string()),
    }
}

fn parse_python_value(s: &str) -> CyberPiValue {
    let s = s.trim();

    // Boolean
    if s == "True" { return CyberPiValue::Bool(true); }
    if s == "False" { return CyberPiValue::Bool(false); }
    if s == "None" { return CyberPiValue::None; }

    // String (single or double quoted)
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return CyberPiValue::Text(s[1..s.len()-1].to_string());
    }

    // Integer
    if let Ok(i) = s.parse::<i64>() {
        return CyberPiValue::Int(i);
    }

    // Float
    if let Ok(f) = s.parse::<f64>() {
        return CyberPiValue::Float(f);
    }

    CyberPiValue::Text(s.to_string())
}

// ---------------------------------------------------------------------------
// Legacy compatibility stubs (for code that references old protocol.rs)
// These delegate to the correct f3/f4 protocol functions.
// ---------------------------------------------------------------------------

/// Legacy: motor command. Now generates an f3 script frame.
pub fn motor_cmd(left: i8, right: i8) -> Vec<u8> {
    command(&motor_script(left, right), 0)
}

/// Legacy: LED command. Now generates an f3 script frame.
pub fn led_cmd(rgb: [u8; 3]) -> Vec<u8> {
    command(&led_script(rgb[0], rgb[1], rgb[2]), 0)
}

/// Legacy: servo command. Now generates an f3 script frame.
pub fn servo_cmd(port: u8, angle: u8) -> Vec<u8> {
    command(&servo_script(port, angle), 0)
}

/// Legacy: buzzer command. Now generates an f3 script frame.
pub fn buzzer_cmd(frequency: u16, duration_ms: u16) -> Vec<u8> {
    command(&buzzer_script(frequency, duration_ms), 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_frame_structure() {
        let frame = script_frame("cyberpi.get_bri()", 1, MODE_RUN_WITH_RESPONSE);

        // Header
        assert_eq!(frame[0], 0xf3);
        // Footer
        assert_eq!(*frame.last().unwrap(), 0xf4);
        // Type
        assert_eq!(frame[4], TYPE_SCRIPT);
        // Mode
        assert_eq!(frame[5], MODE_RUN_WITH_RESPONSE);
        // Idx
        assert_eq!(frame[6], 1); // idx_lo
        assert_eq!(frame[7], 0); // idx_hi
    }

    #[test]
    fn test_script_frame_matches_python() {
        // From Python test output:
        // TX[0]: f3 0a 17 00 28 01 01 00 11 00 63 79 62 65 72 70 69 2e 67 65 74 5f 62 72 69 28 29 84 f4
        let frame = script_frame("cyberpi.get_bri()", 1, MODE_RUN_WITH_RESPONSE);
        assert_eq!(frame[0], 0xf3);
        assert_eq!(frame[1], 0x0a); // hdr_check
        assert_eq!(frame[2], 0x17); // datalen_lo = 23
        assert_eq!(frame[3], 0x00); // datalen_hi = 0
        assert_eq!(frame[4], 0x28); // TYPE_SCRIPT
        assert_eq!(frame[5], 0x01); // MODE_RUN_WITH_RESPONSE
        assert_eq!(frame[6], 0x01); // idx_lo
        assert_eq!(frame[7], 0x00); // idx_hi
        assert_eq!(frame[8], 0x11); // script_len_lo = 17 = len("cyberpi.get_bri()")
        assert_eq!(frame[9], 0x00); // script_len_hi = 0
        // Script bytes
        assert_eq!(&frame[10..27], b"cyberpi.get_bri()");
        // Checksum
        assert_eq!(frame[27], 0x84);
        // Footer
        assert_eq!(frame[28], 0xf4);
    }

    #[test]
    fn test_online_mode_frame() {
        let frame = online_mode_frame();
        assert_eq!(frame, vec![0xf3, 0xf6, 0x03, 0x00, 0x0d, 0x00, 0x01, 0x0e, 0xf4]);
    }

    #[test]
    fn test_parser_script_response() {
        // Real response from CyberPi: brightness = 75
        let raw = [0xf3, 0x03, 0x10, 0x00, 0x28, 0x01, 0x01, 0x00,
                   0x0a, 0x00, 0x7b, 0x22, 0x72, 0x65, 0x74, 0x22,
                   0x3a, 0x37, 0x35, 0x7d, 0x61, 0xf4];

        let mut parser = F3Parser::new();
        let responses = parser.feed(&raw);

        assert_eq!(responses.len(), 1);
        assert!(parser.ready);
        assert_eq!(responses[0].idx, 1);
        assert_eq!(responses[0].frame_type, TYPE_SCRIPT);

        match &responses[0].value {
            CyberPiValue::Int(v) => assert_eq!(*v, 75),
            _ => panic!("Expected Int(75), got {:?}", responses[0].value),
        }
    }

    #[test]
    fn test_parser_float_response() {
        // Real response: accel_z = -9.6 → {"ret":-9.6}
        let raw = [0xf3, 0x05, 0x12, 0x00, 0x28, 0x01, 0x6c, 0x00,
                   0x0c, 0x00, 0x7b, 0x22, 0x72, 0x65, 0x74, 0x22,
                   0x3a, 0x2d, 0x39, 0x2e, 0x36, 0x7d, 0x2c, 0xf4];

        let mut parser = F3Parser::new();
        let responses = parser.feed(&raw);

        assert_eq!(responses.len(), 1);
        match &responses[0].value {
            CyberPiValue::Float(v) => assert!((*v - (-9.6)).abs() < 0.01),
            _ => panic!("Expected Float(-9.6), got {:?}", responses[0].value),
        }
    }

    #[test]
    fn test_parser_string_response() {
        // Real response: firmware = "44.01.009" → {"ret":"44.01.009"}
        let raw = [0xf3, 0x0c, 0x19, 0x00, 0x28, 0x01, 0x70, 0x00,
                   0x13, 0x00, 0x7b, 0x22, 0x72, 0x65, 0x74, 0x22,
                   0x3a, 0x22, 0x34, 0x34, 0x2e, 0x30, 0x31, 0x2e,
                   0x30, 0x30, 0x39, 0x22, 0x7d, 0x6f, 0xf4];

        let mut parser = F3Parser::new();
        let responses = parser.feed(&raw);

        assert_eq!(responses.len(), 1);
        match &responses[0].value {
            CyberPiValue::Text(s) => assert_eq!(s, "44.01.009"),
            _ => panic!("Expected Text, got {:?}", responses[0].value),
        }
    }

    #[test]
    fn test_parser_multiple_frames() {
        // Two frames concatenated (brightness responses)
        let raw = [
            // Frame 1: {"ret":75} idx=1
            0xf3, 0x03, 0x10, 0x00, 0x28, 0x01, 0x01, 0x00,
            0x0a, 0x00, 0x7b, 0x22, 0x72, 0x65, 0x74, 0x22,
            0x3a, 0x37, 0x35, 0x7d, 0x61, 0xf4,
            // Frame 2: {"ret":74} idx=2
            0xf3, 0x03, 0x10, 0x00, 0x28, 0x01, 0x02, 0x00,
            0x0a, 0x00, 0x7b, 0x22, 0x72, 0x65, 0x74, 0x22,
            0x3a, 0x37, 0x34, 0x7d, 0x61, 0xf4,
        ];

        let mut parser = F3Parser::new();
        let responses = parser.feed(&raw);

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].idx, 1);
        assert_eq!(responses[1].idx, 2);
    }

    #[test]
    fn test_parser_with_noise() {
        // Garbage bytes before a valid frame
        let mut raw = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
        // Then a valid frame: {"ret":100} idx=0x66
        raw.extend_from_slice(&[
            0xf3, 0x04, 0x11, 0x00, 0x28, 0x01, 0x66, 0x00,
            0x0b, 0x00, 0x7b, 0x22, 0x72, 0x65, 0x74, 0x22,
            0x3a, 0x31, 0x30, 0x30, 0x7d, 0xec, 0xf4,
        ]);

        let mut parser = F3Parser::new();
        let responses = parser.feed(&raw);

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].idx, 0x66);
        match &responses[0].value {
            CyberPiValue::Int(v) => assert_eq!(*v, 100),
            _ => panic!("Expected Int(100), got {:?}", responses[0].value),
        }
    }

    #[test]
    fn test_parse_ret_value_int() {
        match parse_ret_value(r#"{"ret":42}"#) {
            CyberPiValue::Int(v) => assert_eq!(v, 42),
            other => panic!("Expected Int(42), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ret_value_float() {
        match parse_ret_value(r#"{"ret":-9.6}"#) {
            CyberPiValue::Float(v) => assert!((v - (-9.6)).abs() < 0.01),
            other => panic!("Expected Float(-9.6), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ret_value_string() {
        match parse_ret_value(r#"{"ret":"hello"}"#) {
            CyberPiValue::Text(s) => assert_eq!(s, "hello"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ret_value_bool() {
        match parse_ret_value(r#"{"ret":true}"#) {
            CyberPiValue::Bool(b) => assert!(b),
            other => panic!("Expected Bool(true), got {:?}", other),
        }
    }

    #[test]
    fn test_motor_cmd_generates_f3_frame() {
        let cmd = motor_cmd(50, -50);
        assert_eq!(cmd[0], 0xf3);
        assert_eq!(*cmd.last().unwrap(), 0xf4);
        assert_eq!(cmd[4], TYPE_SCRIPT);
    }

    #[test]
    fn test_led_cmd_generates_f3_frame() {
        let cmd = led_cmd([255, 0, 0]);
        assert_eq!(cmd[0], 0xf3);
        assert_eq!(*cmd.last().unwrap(), 0xf4);
    }
}
