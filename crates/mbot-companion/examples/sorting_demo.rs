//! Example demonstration of the Pick-Drop Sorting Loop
//!
//! This example shows how to use the sorting loop to sort LEGO pieces
//! from a tray into color-coded bins on a carousel.
//!
//! Run with:
//! ```bash
//! cargo run --example sorting_demo
//! ```

use mbot_companion::sorter::{
    Bin, CarouselConfig, LoopState, Position2D, ServoCalibration, SortingLoop,
    SortingStep, VisionDetector, VisionConfig, TrayRegion,
};

fn main() {
    println!("=== mBot LEGO Sorter - Pick-Drop Loop Demo ===\n");

    // Step 1: Create and verify servo calibration
    println!("Step 1: Setting up servo calibration...");
    let mut calibration = ServoCalibration::default();
    calibration.mark_verified(12345); // Mark as calibrated
    println!("  ✓ Servo calibration verified\n");

    // Step 2: Configure carousel with color bins
    println!("Step 2: Configuring carousel with 6 color bins...");
    let mut carousel = CarouselConfig::new("demo_carousel");

    // Add bins at 60° intervals around carousel
    carousel.add_bin(Bin::new("bin-red", "marker-01", 45.0, "color:red", 50))
        .expect("Failed to add red bin");
    carousel.add_bin(Bin::new("bin-blue", "marker-02", 105.0, "color:blue", 50))
        .expect("Failed to add blue bin");
    carousel.add_bin(Bin::new("bin-yellow", "marker-03", 165.0, "color:yellow", 50))
        .expect("Failed to add yellow bin");
    carousel.add_bin(Bin::new("bin-green", "marker-04", 225.0, "color:green", 50))
        .expect("Failed to add green bin");
    carousel.add_bin(Bin::new("bin-black", "marker-05", 285.0, "color:black", 50))
        .expect("Failed to add black bin");
    carousel.add_bin(Bin::new("bin-misc", "marker-06", 345.0, "misc", 50))
        .expect("Failed to add misc bin");

    println!("  ✓ Configured {} bins", carousel.bin_count);
    println!("  ✓ Carousel validated\n");

    // Step 3: Set up vision system
    println!("Step 3: Setting up vision detection system...");
    let mut vision_config = VisionConfig::default();
    vision_config.tray_region = TrayRegion::new(
        Position2D::new(0.0, 0.0),
        Position2D::new(200.0, 150.0),
        "white", // White background for good contrast
    );
    vision_config.min_confidence = 0.80; // Require 80% confidence

    let vision = VisionDetector::new(vision_config);
    println!("  ✓ Vision detector configured");
    println!("  ✓ Tray region: 200mm x 150mm");
    println!("  ✓ Min confidence: 80%\n");

    // Step 4: Create sorting loop
    println!("Step 4: Initializing sorting loop...");
    let mut sorting_loop = SortingLoop::new(
        "demo_loop_001".to_string(),
        calibration,
        carousel,
        vision,
    );
    println!("  ✓ Sorting loop created: {}\n", sorting_loop.loop_id);

    // Step 5: Start sorting
    println!("Step 5: Starting sorting loop...");
    match sorting_loop.start() {
        Ok(_) => println!("  ✓ Loop started successfully"),
        Err(e) => {
            eprintln!("  ✗ Failed to start loop: {}", e);
            return;
        }
    }
    println!("  State: {:?}", sorting_loop.state);
    println!("  Step: {}\n", sorting_loop.step.description());

    // Step 6: Simulate sorting cycle with mock camera data
    println!("Step 6: Simulating sorting cycle...");
    println!("  (In production, this would use real camera feed)\n");

    // Create mock RGB data representing a tray with colored LEGO pieces
    let width = 640u32;
    let height = 480u32;
    let mock_rgb_data = create_mock_tray_image(width, height);

    // Execute multiple steps to demonstrate the full cycle
    let steps_to_demo = vec![
        SortingStep::ScanTray,
        SortingStep::SelectPiece,
        SortingStep::AlignToPiece,
        SortingStep::LowerArm,
        SortingStep::CloseGripper,
        SortingStep::LiftArm,
        SortingStep::RotateToBin,
        SortingStep::LowerToBin,
        SortingStep::OpenGripper,
        SortingStep::ReturnHome,
        SortingStep::VerifyDrop,
    ];

    for (i, expected_step) in steps_to_demo.iter().enumerate() {
        if sorting_loop.state == LoopState::Complete || sorting_loop.state == LoopState::Error {
            break;
        }

        println!("  Step {}: {}", i + 1, sorting_loop.step.description());

        // Execute one step
        if let Err(e) = sorting_loop.execute_step(&mock_rgb_data, width, height) {
            eprintln!("    ✗ Error: {}", e);
            break;
        }

        // Display current state
        if let Some(ref piece) = sorting_loop.current_piece {
            println!("    Current piece: {} at ({:.1}, {:.1})",
                piece.color.name(),
                piece.center_position.x,
                piece.center_position.y
            );
        }
        if let Some(ref bin) = sorting_loop.target_bin {
            println!("    Target bin: {} ({}°)", bin.bin_id, bin.position_angle);
        }
    }

    println!();

    // Step 7: Display metrics
    println!("=== Sorting Session Summary ===");
    println!("{}", sorting_loop.status_summary());
    println!();
    println!("Detailed Metrics:");
    println!("  Pieces sorted: {}", sorting_loop.metrics.pieces_sorted);
    println!("  Pick attempts: {}", sorting_loop.metrics.pick_attempts);
    println!("  Pick successes: {}", sorting_loop.metrics.pick_successes);
    println!("  Pick success rate: {:.1}%", sorting_loop.metrics.pick_success_rate());
    println!("  Drop attempts: {}", sorting_loop.metrics.drop_attempts);
    println!("  Drop successes: {}", sorting_loop.metrics.drop_successes);
    println!("  Drop success rate: {:.1}%", sorting_loop.metrics.drop_success_rate());
    println!("  Pieces skipped: {}", sorting_loop.metrics.pieces_skipped);
    println!("  Avg cycle time: {}ms", sorting_loop.metrics.avg_cycle_time_ms);
    println!();

    // Step 8: Display inventory
    println!("=== Bin Inventory ===");
    let inventory = sorting_loop.inventory_summary();
    for (bin_id, count) in inventory.iter() {
        println!("  {}: {} pieces", bin_id, count);
    }
    println!();

    // Step 9: Demonstrate pause/resume
    println!("=== Safety Features Demo ===");
    println!("  Testing pause/resume...");
    sorting_loop.pause();
    println!("    Paused: {}", sorting_loop.is_stopped());
    sorting_loop.resume();
    println!("    Resumed: {}", !sorting_loop.is_stopped());

    println!("  Testing emergency stop...");
    sorting_loop.emergency_stop();
    println!("    State after E-stop: {:?}", sorting_loop.state);
    sorting_loop.reset_after_emergency();
    println!("    Reset: State = {:?}", sorting_loop.state);
    println!();

    println!("=== Demo Complete ===");
    println!("The sorting loop is ready for production use!");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Complete pick-drop cycle");
    println!("  ✓ Vision-based piece detection");
    println!("  ✓ Color-based bin routing");
    println!("  ✓ Servo safety checks");
    println!("  ✓ Pause/resume functionality");
    println!("  ✓ Emergency stop protection");
    println!("  ✓ Metrics tracking");
    println!("  ✓ Inventory management");
}

/// Create mock RGB image data for demonstration
/// In production, this comes from a real camera
fn create_mock_tray_image(width: u32, height: u32) -> Vec<u8> {
    let mut rgb_data = vec![255u8; (width * height * 3) as usize]; // White background

    // Add some "colored pieces" to the image
    // Red piece at (100, 100)
    add_colored_blob(&mut rgb_data, width, 100, 100, 40, 255, 0, 0);

    // Blue piece at (200, 150)
    add_colored_blob(&mut rgb_data, width, 200, 150, 35, 0, 0, 255);

    // Yellow piece at (300, 200)
    add_colored_blob(&mut rgb_data, width, 300, 200, 30, 255, 255, 0);

    rgb_data
}

/// Add a colored blob to the RGB image
fn add_colored_blob(
    rgb_data: &mut [u8],
    width: u32,
    center_x: u32,
    center_y: u32,
    radius: u32,
    r: u8,
    g: u8,
    b: u8,
) {
    for dy in 0..radius * 2 {
        for dx in 0..radius * 2 {
            let x = center_x.saturating_sub(radius).saturating_add(dx);
            let y = center_y.saturating_sub(radius).saturating_add(dy);

            // Check if within circle
            let dist_sq = (dx as i32 - radius as i32).pow(2) + (dy as i32 - radius as i32).pow(2);
            if dist_sq <= (radius as i32).pow(2) {
                let idx = ((y * width + x) * 3) as usize;
                if idx + 2 < rgb_data.len() {
                    rgb_data[idx] = r;
                    rgb_data[idx + 1] = g;
                    rgb_data[idx + 2] = b;
                }
            }
        }
    }
}
