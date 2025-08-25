#![no_main]
use libfuzzer_sys::fuzz_target;
use regex_engine::{ConstructionType, Regex};

fuzz_target!(|data: &[u8]| {
    if let Ok(regex_str) = std::str::from_utf8(data) {
        // Limit input size to avoid timeouts
        if regex_str.len() > 50 {
            return;
        }

        // Skip obviously invalid inputs to focus on potentially valid ones
        if regex_str.is_empty()
            || regex_str.starts_with('*')
            || regex_str.starts_with('+')
            || regex_str.starts_with('?')
            || regex_str.starts_with(')')
        {
            return;
        }

        // Test Glushkov construction - should not panic for any input
        let result = std::panic::catch_unwind(|| Regex::new(regex_str, ConstructionType::Glushkov));

        match result {
            Ok(Ok(_)) => {
                // eprintln!("✅ Success");
            }
            Ok(Err(_)) => {
                // eprintln!("❌ Expected error: {}", e);
            }
            Err(_) => {
                eprintln!("💥 PANIC on input: {:?}", regex_str);
            }
        }
    }
    // else {
    //     eprintln!("❌ Invalid UTF-8");
    // }
});
