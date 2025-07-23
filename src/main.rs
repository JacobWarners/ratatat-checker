// main.rs

use inputbot::KeybdKey;
use std::env;
use std::process::Command;
use std::sync::{Arc, Mutex};

fn main() {
    // Define the sequence to play the song.
    let play_sequence = vec![
        KeybdKey::RKey, KeybdKey::AKey, KeybdKey::TKey, KeybdKey::AKey,
        KeybdKey::TKey, KeybdKey::AKey, KeybdKey::TKey,
    ];

    // Define the new sequence to kill the music player.
    let kill_sequence = vec![
        KeybdKey::EscapeKey,
        KeybdKey::EscapeKey,
        KeybdKey::EscapeKey,
    ];

    let recent_keys = Arc::new(Mutex::new(Vec::<KeybdKey>::new()));

    let callback = move |key: KeybdKey| {
        // Use a block to ensure the lock is dropped before any commands run.
        let (should_play, should_kill) = {
            let mut keys = match recent_keys.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("Recovering from poisoned mutex");
                    poisoned.into_inner()
                }
            };
            keys.push(key);

            // Keep the buffer at a reasonable size (the length of the longest sequence).
            let max_len = play_sequence.len();
            if keys.len() > max_len {
                keys.remove(0);
            }
            
            println!("Current sequence: {:?}", *keys);

            // Check if the end of our recent keys matches the play sequence.
            if keys.ends_with(&play_sequence) {
                keys.clear();
                (true, false) // Signal to play music.
            } 
            // Check if the end of our recent keys matches the kill sequence.
            else if keys.ends_with(&kill_sequence) {
                keys.clear();
                (false, true) // Signal to kill the player.
            } else {
                (false, false)
            }
        }; // Mutex lock is dropped here.

        if should_play {
            println!("'ratatat' sequence detected! Attempting to play song...");
            let song_path = match env::var("RATATAT_SONG_PATH") {
                Ok(path) => path,
                Err(_) => {
                    eprintln!("Error: RATATAT_SONG_PATH environment variable not set.");
                    return;
                }
            };

            if let Err(e) = Command::new("mpg123").arg(song_path).spawn() {
                eprintln!("Failed to play song: {}", e);
            }
        }

        if should_kill {
            println!("Kill sequence detected! Stopping audio players...");
            // Run the pkill command to stop mpg123.
            if let Err(e) = Command::new("pkill").arg("mpg123").spawn() {
                eprintln!("Failed to run pkill on mpg123: {}", e);
            }
            // Also run pkill on paplay.
            if let Err(e) = Command::new("pkill").arg("paplay").spawn() {
                eprintln!("Failed to run pkill on paplay: {}", e);
            }
        }
    };

    KeybdKey::bind_all(callback);
    println!("Listener started. Type 'ratatat' to play a song. Press ESC 3 times to stop.");
    inputbot::handle_input_events();
}

