use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tungstenite::Message as WsMessage;

const ANALOG_DEADZONE: u16 = 10;
const ANALOG_MAX: f32 = 1550.0;

/// Thread-safe analog keyboard state.
/// Provides 0.0..1.0 values for each HID scancode (256 slots).
#[derive(Clone)]
pub struct AnalogKeyboard {
    vid: u16,
    pid: u16,
    values: Arc<Mutex<[f32; 256]>>,
    active: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
}

impl AnalogKeyboard {
    pub fn new(vid: u16, pid: u16) -> Self {
        Self {
            vid,
            pid,
            values: Arc::new(Mutex::new([0.0f32; 256])),
            active: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Starting...".into())),
        }
    }

    /// Spawn background thread that detects keyboard and starts WebHID bridge.
    pub fn start(&self) {
        let kb = self.clone();
        thread::spawn(move || hid_thread(&kb));
    }

    /// Snapshot of all 256 analog values (0.0 = released, 1.0 = fully pressed).
    pub fn values(&self) -> [f32; 256] {
        self.values.lock().map(|v| *v).unwrap_or([0.0; 256])
    }

    /// Single key value by HID scancode.
    pub fn value(&self, scancode: u8) -> f32 {
        self.values
            .lock()
            .map(|v| v[scancode as usize])
            .unwrap_or(0.0)
    }

    /// Set values directly (for fallback digital input).
    pub fn set_values(&self, vals: &[f32; 256]) {
        if let Ok(mut v) = self.values.lock() {
            *v = *vals;
        }
    }

    /// True when analog HID data is streaming.
    pub fn is_active(&self) -> bool {
        self.active.lock().map(|v| *v).unwrap_or(false)
    }

    /// Current human-readable status message.
    pub fn status(&self) -> String {
        self.status.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn vid(&self) -> u16 {
        self.vid
    }

    pub fn pid(&self) -> u16 {
        self.pid
    }
}

// ─── internals ───────────────────────────────────────────────────────────

fn set_status(kb: &AnalogKeyboard, msg: &str) {
    if let Ok(mut m) = kb.status.lock() {
        *m = msg.into();
    }
    log::info!("[HID] {msg}");
}

fn hid_thread(kb: &AnalogKeyboard) {
    loop {
        let found = hidapi::HidApi::new()
            .map(|api| {
                api.device_list()
                    .any(|d| d.vendor_id() == kb.vid && d.product_id() == kb.pid)
            })
            .unwrap_or(false);

        if !found {
            set_status(kb, "Keyboard not found - plug it in");
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        set_status(kb, "Keyboard detected - launching Chrome bridge...");
        start_webhid_bridge(kb);
        thread::sleep(Duration::from_secs(2));
    }
}

fn parse_analog_input(data: &[u8], kb: &AnalogKeyboard) {
    if data.len() < 6 || data[0] != 0xA0 {
        return;
    }

    let key_idx = data[3] as usize;
    let raw = ((data[4] as u16) << 8) | (data[5] as u16);

    let value = if raw <= ANALOG_DEADZONE {
        0.0
    } else {
        ((raw - ANALOG_DEADZONE) as f32 / ANALOG_MAX).clamp(0.0, 1.0)
    };

    let Ok(mut tgt) = kb.values.lock() else {
        return;
    };
    if key_idx < 256 {
        tgt[key_idx] = value;
    }
}

fn bridge_html(ws_port: u16, vid: u16, pid: u16) -> String {
    include_str!("bridge.html")
        .replace("__WS_PORT__", &ws_port.to_string())
        .replace("__VID__", &format!("0x{:04X}", vid))
        .replace("__PID__", &format!("0x{:04X}", pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keyboard_initial_state() {
        let kb = AnalogKeyboard::new(0x1234, 0x5678);
        assert_eq!(kb.vid(), 0x1234);
        assert_eq!(kb.pid(), 0x5678);
        assert!(!kb.is_active());
        assert_eq!(kb.status(), "Starting...");
        assert_eq!(kb.values(), [0.0f32; 256]);
    }

    #[test]
    fn set_and_read_values() {
        let kb = AnalogKeyboard::new(0, 0);
        let mut vals = [0.0f32; 256];
        vals[0x04] = 0.75;
        vals[0x2C] = 1.0;

        kb.set_values(&vals);

        assert_eq!(kb.value(0x04), 0.75);
        assert_eq!(kb.value(0x2C), 1.0);
        assert_eq!(kb.value(0x00), 0.0);
        assert_eq!(kb.values()[0x04], 0.75);
    }

    #[test]
    fn set_values_overwrites_completely() {
        let kb = AnalogKeyboard::new(0, 0);
        let mut v1 = [0.0f32; 256];
        v1[10] = 1.0;
        kb.set_values(&v1);

        let v2 = [0.0f32; 256];
        kb.set_values(&v2);

        assert_eq!(kb.value(10), 0.0);
    }

    #[test]
    fn parse_analog_valid_keypress() {
        let kb = AnalogKeyboard::new(0, 0);
        // report id=0xA0, padding, padding, key_idx=0x04, raw_hi=0x03, raw_lo=0x00 → raw=768
        let data = [0xA0, 0x00, 0x00, 0x04, 0x03, 0x00];
        parse_analog_input(&data, &kb);

        let v = kb.value(0x04);
        let expected = (768.0 - ANALOG_DEADZONE as f32) / ANALOG_MAX;
        assert!(
            (v - expected).abs() < 0.001,
            "got {v}, expected ~{expected}"
        );
    }

    #[test]
    fn parse_analog_below_deadzone_is_zero() {
        let kb = AnalogKeyboard::new(0, 0);
        // raw = 5, below ANALOG_DEADZONE (10)
        let data = [0xA0, 0x00, 0x00, 0x04, 0x00, 0x05];
        parse_analog_input(&data, &kb);
        assert_eq!(kb.value(0x04), 0.0);
    }

    #[test]
    fn parse_analog_at_deadzone_is_zero() {
        let kb = AnalogKeyboard::new(0, 0);
        // raw = ANALOG_DEADZONE exactly
        let data = [0xA0, 0x00, 0x00, 0x04, 0x00, ANALOG_DEADZONE as u8];
        parse_analog_input(&data, &kb);
        assert_eq!(kb.value(0x04), 0.0);
    }

    #[test]
    fn parse_analog_clamped_to_one() {
        let kb = AnalogKeyboard::new(0, 0);
        // raw = 0xFFFF = 65535, way above ANALOG_MAX → should clamp to 1.0
        let data = [0xA0, 0x00, 0x00, 0x10, 0xFF, 0xFF];
        parse_analog_input(&data, &kb);
        assert_eq!(kb.value(0x10), 1.0);
    }

    #[test]
    fn parse_analog_ignores_wrong_report_id() {
        let kb = AnalogKeyboard::new(0, 0);
        let data = [0x01, 0x00, 0x00, 0x04, 0x03, 0x00];
        parse_analog_input(&data, &kb);
        assert_eq!(kb.value(0x04), 0.0);
    }

    #[test]
    fn parse_analog_ignores_short_data() {
        let kb = AnalogKeyboard::new(0, 0);
        let data = [0xA0, 0x00, 0x00];
        parse_analog_input(&data, &kb);
        // no crash, values stay zero
        assert_eq!(kb.values(), [0.0f32; 256]);
    }
}

fn start_webhid_bridge(kb: &AnalogKeyboard) {
    use std::net::TcpListener;

    let http_listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            set_status(kb, &format!("HTTP bind: {e}"));
            return;
        }
    };
    let ws_listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            set_status(kb, &format!("WS bind: {e}"));
            return;
        }
    };

    let http_port = http_listener.local_addr().unwrap().port();
    let ws_port = ws_listener.local_addr().unwrap().port();
    let html = Arc::new(bridge_html(ws_port, kb.vid, kb.pid));

    let h = html.clone();
    thread::spawn(move || {
        use std::io::{Read, Write};
        for stream in http_listener.incoming().flatten() {
            let mut s = stream;
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = [0u8; 2048];
            let _ = s.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html;charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                h.len(), &*h
            );
            let _ = s.write_all(resp.as_bytes());
        }
    });

    let url = format!("http://127.0.0.1:{http_port}");
    set_status(kb, &format!("Open Chrome -> {url}"));

    if cfg!(target_os = "macos") {
        let _ = std::process::Command::new("open")
            .args(["-a", "Google Chrome", &url])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    } else {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }

    ws_listener.set_nonblocking(true).ok();

    loop {
        set_status(kb, "Waiting for Chrome connection...");
        if let Ok(mut h) = kb.active.lock() {
            *h = false;
        }

        let stream = loop {
            match ws_listener.accept() {
                Ok((s, _)) => {
                    s.set_nonblocking(false).ok();
                    break s;
                }
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        };

        let mut websocket = match tungstenite::accept(stream) {
            Ok(ws) => ws,
            Err(_) => continue,
        };

        set_status(kb, "Chrome connected - click Connect in browser");
        let mut got_analog = false;

        loop {
            match websocket.read() {
                Ok(WsMessage::Binary(data)) => {
                    if data.len() < 3 {
                        continue;
                    }
                    let payload = &data[2..];

                    if data[0] == 0x03 {
                        if !got_analog {
                            got_analog = true;
                            if let Ok(mut h) = kb.active.lock() {
                                *h = true;
                            }
                            set_status(kb, "Analog active!");
                        }
                        parse_analog_input(payload, kb);

                        let pressed = kb
                            .values
                            .lock()
                            .map(|t| t.iter().filter(|&&v| v > 0.01).count())
                            .unwrap_or(0);
                        if let Ok(mut m) = kb.status.lock() {
                            *m = format!("Analog active! ({pressed} keys)");
                        }
                    }
                }
                Ok(WsMessage::Close(_)) | Err(_) => break,
                _ => {}
            }
        }

        if let Ok(mut h) = kb.active.lock() {
            *h = false;
        }
        set_status(kb, "Chrome disconnected - reconnecting...");
        thread::sleep(Duration::from_millis(500));
    }
}
