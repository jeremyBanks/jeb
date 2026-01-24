#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(4)];

    // Try to decode as UTF-8
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    // Try to parse as JSON
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };

    // Format with Debug (this should never panic for valid JSON)
    let _ = format!("{:?}", value);
});
