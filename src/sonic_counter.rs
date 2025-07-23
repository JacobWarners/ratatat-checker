use inputbot::KeybdKey;
use std::fs;
use std::process::Command;

// --- Configuration for this module ---
const COUNT_FILE: &str = "/tmp/waybar_counter";
const STATE_FILE: &str = "/tmp/sonic_state";
const WAYBAR_WORKSPACE_SIGNAL: i32 = 1;
const SYMBOL_COUNT: u8 = 3;

/// Holds the state for just the sonic counter.
#[derive(Clone)]
pub struct SonicState {
    letter_count: u8,
    main_counter: u8,
    sonic_state: u8,
}

impl SonicState {
    /// Creates a new, default state for the counter.
    pub fn new() -> Self {
        SonicState {
            letter_count: 0,
            main_counter: 0,
            sonic_state: 0,
        }
    }

    /// On startup, ensure our state files are created with default values.
    pub fn initialize_files() {
        fs::write(COUNT_FILE, "0").expect("Failed to initialize counter file.");
        fs::write(STATE_FILE, "0").expect("Failed to initialize state file.");
        // Print the initial counter value so Waybar shows "0" on start.
        println!("0");
    }
}

/// A helper function to check if a pressed key is an alphabet character.
fn is_letter(key: KeybdKey) -> bool {
    matches!(key, KeybdKey::AKey..=KeybdKey::ZKey)
}

/// The main handler function for this module. It's called on every key press.
/// It returns `true` if it handled the key press, and `false` otherwise.
/// This tells the main event loop whether to continue processing the key.
pub fn handle_key_press(key: KeybdKey, state: &mut SonicState) -> bool {
    // --- Trigger Logic ---
    // Check for the trigger: Backslash key is pressed AND the counter is 50.
    if key == KeybdKey::BackslashKey && state.main_counter >= 50 {
        // Cycle to the next symbol state
        state.sonic_state = (state.sonic_state + 1) % SYMBOL_COUNT;
        fs::write(STATE_FILE, state.sonic_state.to_string()).ok();

        // Reset the counter
        state.main_counter = 0;
        state.letter_count = 0;
        fs::write(COUNT_FILE, "0").ok();
        println!("0"); // Print the new value for Waybar

        // Signal Waybar to refresh
        let signal = format!("-RTMIN+{}", WAYBAR_WORKSPACE_SIGNAL);
        if let Err(e) = Command::new("pkill").args(["-x", "waybar", &signal]).status() {
            eprintln!("Failed to signal Waybar: {}", e);
        }
        return true; // We handled this key, stop further processing.

    // --- Counting Logic ---
    } else if is_letter(key) {
        state.letter_count += 1;

        if state.letter_count >= 5 {
            state.letter_count = 0;
            if state.main_counter < 50 {
                state.main_counter += 1;
                let counter_str = state.main_counter.to_string();
                fs::write(COUNT_FILE, &counter_str).ok();
                println!("{}", counter_str);
            }
        }
        return true; // We handled this key, stop further processing.
    }

    // If it wasn't a letter or the trigger, we didn't handle it.
    false
}
