//! Inventory Tracking System Demo
//!
//! This example demonstrates the inventory tracking features for LEGO sorting:
//! - Real-time bin counts
//! - Persistent storage
//! - Capacity warnings
//! - Search functionality
//! - Event history

use mbot_core::helperbot::{BinConfig, InventoryTracker};

fn main() {
    println!("=== mBot Inventory Tracking System Demo ===\n");

    // Initialize inventory tracker with 90% capacity warning threshold
    let mut tracker = InventoryTracker::new(90.0);

    // Configure bins
    println!("1. Configuring bins...");
    tracker.add_bin(BinConfig::new(
        "bin-01".to_string(),
        "Red Parts".to_string(),
        "color".to_string(),
        50,
    ));
    tracker.add_bin(BinConfig::new(
        "bin-02".to_string(),
        "Blue Parts".to_string(),
        "color".to_string(),
        50,
    ));
    tracker.add_bin(BinConfig::new(
        "bin-03".to_string(),
        "Small Pieces".to_string(),
        "size".to_string(),
        100,
    ));
    println!("   ✓ Configured 3 bins\n");

    // Start sorting session
    println!("2. Starting sorting session...");
    tracker.start_session("demo-session-001".to_string());
    println!("   ✓ Session started: demo-session-001\n");

    // Simulate sorting pieces
    println!("3. Sorting pieces...");
    let timestamp = get_timestamp_us();

    // Sort red pieces
    for i in 0..15 {
        tracker.record_sorted(
            "bin-01".to_string(),
            "red".to_string(),
            0.95,
            timestamp + i * 100,
        );
    }
    println!("   ✓ Sorted 15 red pieces to bin-01");

    // Sort blue pieces
    for i in 0..8 {
        tracker.record_sorted(
            "bin-02".to_string(),
            "blue".to_string(),
            0.92,
            timestamp + 2000 + i * 100,
        );
    }
    println!("   ✓ Sorted 8 blue pieces to bin-02");

    // Sort mixed pieces to bin-03
    tracker.record_sorted("bin-03".to_string(), "yellow".to_string(), 0.88, timestamp + 3000);
    tracker.record_sorted("bin-03".to_string(), "green".to_string(), 0.90, timestamp + 3100);
    tracker.record_sorted("bin-03".to_string(), "yellow".to_string(), 0.91, timestamp + 3200);
    println!("   ✓ Sorted 3 small pieces to bin-03\n");

    // Query inventory
    println!("4. Querying inventory...");
    println!("   Total pieces across all bins: {}", tracker.get_total_count());
    println!("   Red pieces: {}", tracker.get_color_count("red"));
    println!("   Blue pieces: {}", tracker.get_color_count("blue"));
    println!("   Yellow pieces: {}\n", tracker.get_color_count("yellow"));

    // View bin contents
    println!("5. Bin contents:");
    let bin1_contents = tracker.get_bin_contents("bin-01");
    for count in &bin1_contents {
        println!(
            "   bin-01 ({}): {} pieces @ {:.1}% confidence",
            count.color,
            count.count,
            count.confidence * 100.0
        );
    }

    let bin3_contents = tracker.get_bin_contents("bin-03");
    for count in &bin3_contents {
        println!(
            "   bin-03 ({}): {} pieces @ {:.1}% confidence",
            count.color,
            count.count,
            count.confidence * 100.0
        );
    }
    println!();

    // Search inventory
    println!("6. Searching inventory...");
    let yellow_results = tracker.search("yellow");
    println!("   Search for 'yellow': {} results", yellow_results.len());
    for result in &yellow_results {
        println!(
            "     - {} ({}): {} pieces",
            result.bin_name, result.bin_id, result.count
        );
    }
    println!();

    // Manual adjustment
    println!("7. Manual adjustment...");
    println!("   User manually recounts bin-01: found 14 pieces (not 15)");
    tracker.adjust_bin("bin-01".to_string(), "red".to_string(), 14, timestamp + 5000);
    println!("   ✓ Adjusted bin-01 red count to 14\n");

    // Check capacity warnings
    println!("8. Checking capacity warnings...");
    let warnings = tracker.check_capacity_warnings();
    if warnings.is_empty() {
        println!("   ✓ No capacity warnings\n");
    } else {
        for (bin_name, fill_pct) in &warnings {
            println!("   ⚠️  {} is {:.1}% full", bin_name, fill_pct);
        }
        println!();
    }

    // Generate summary
    println!("9. Inventory summary:");
    let summary = tracker.generate_summary(timestamp + 6000);
    println!("   Session: {:?}", summary.sorting_session);
    println!("   Total pieces: {}", summary.total_pieces);
    println!("   Bins:");
    for bin_summary in &summary.bins {
        println!(
            "     - {} ({}): {} / {} pieces ({:.1}% full)",
            bin_summary.bin_name,
            bin_summary.bin_id,
            bin_summary.piece_count,
            bin_summary.capacity,
            bin_summary.fill_percentage
        );
        if !bin_summary.top_colors.is_empty() {
            print!("       Top colors: ");
            for (i, color_count) in bin_summary.top_colors.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{} ({})", color_count.color, color_count.count);
            }
            println!();
        }
    }
    println!();

    // View event history
    println!("10. Event history (last 5 events):");
    let events = tracker.get_events();
    let recent_events = if events.len() > 5 {
        &events[events.len() - 5..]
    } else {
        events
    };
    for event in recent_events {
        println!(
            "    [{:?}] {:?} {} piece(s) to {} ({})",
            event.source, event.event_type, event.delta, event.bin_id, event.color
        );
    }
    println!();

    // Persistence demo
    println!("11. Testing persistence (SORT-004)...");
    let json = tracker.to_json().expect("Failed to serialize");
    println!("   ✓ Serialized to JSON ({} bytes)", json.len());

    let restored = InventoryTracker::from_json(&json).expect("Failed to deserialize");
    println!("   ✓ Restored from JSON");
    println!("   Verification: {} pieces (matches original: {})",
        restored.get_total_count(),
        restored.get_total_count() == tracker.get_total_count()
    );
    println!();

    // End session
    println!("12. Ending session...");
    tracker.end_session();
    println!("   ✓ Session ended\n");

    println!("=== Demo Complete ===");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Real-time piece counting");
    println!("  ✓ Multiple bin support");
    println!("  ✓ Confidence tracking");
    println!("  ✓ Manual adjustments");
    println!("  ✓ Search functionality");
    println!("  ✓ Capacity warnings");
    println!("  ✓ Event history");
    println!("  ✓ Persistent storage (SORT-004)");
    println!("  ✓ Offline-first operation (SORT-005)");
}

fn get_timestamp_us() -> u64 {
    // In production, this would use actual system time
    // For demo, return a fixed timestamp
    1700000000000000
}
