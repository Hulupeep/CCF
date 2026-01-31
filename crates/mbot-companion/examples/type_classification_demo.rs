//! Type Classification Demo
//!
//! Demonstrates the coarse type classification system for LEGO pieces.
//! Shows how to classify pieces by type (brick, plate, axle, etc.) and
//! integrate with color detection for combined sorting rules.

use mbot_companion::sorter::{
    BoundingBox, CoarseType, LegoColor, PieceObservation, PieceSize, Position2D,
    TypeClassification, TypeClassifier, TypeSortingConditions, TypeSortingRule,
};

fn main() {
    println!("=== LEGO Type Classification Demo ===\n");

    // Initialize type classifier
    let classifier = TypeClassifier::new(0.70, 5.0); // 70% confidence, 5 pixels/mm

    println!("1. Classifying Individual Pieces:\n");

    // Example 1: Red brick (2x4)
    let brick_obs = create_observation(
        "brick_001",
        LegoColor::Red,
        BoundingBox::new(10, 10, 160, 80), // 32mm x 16mm
    );
    let brick_class = classifier.classify(&brick_obs);
    print_classification("Red 2x4 Brick", &brick_class);

    // Example 2: Blue plate (2x4)
    let plate_obs = create_observation(
        "plate_001",
        LegoColor::Blue,
        BoundingBox::new(10, 100, 200, 40), // 40mm x 8mm (flat)
    );
    let plate_class = classifier.classify(&plate_obs);
    print_classification("Blue 2x4 Plate", &plate_class);

    // Example 3: Black technic axle
    let axle_obs = create_observation(
        "axle_001",
        LegoColor::Black,
        BoundingBox::new(10, 200, 250, 25), // 50mm x 5mm (long and narrow)
    );
    let axle_class = classifier.classify(&axle_obs);
    print_classification("Black Technic Axle", &axle_class);

    // Example 4: Yellow tile
    let tile_obs = create_observation(
        "tile_001",
        LegoColor::Yellow,
        BoundingBox::new(10, 300, 180, 35), // Flat, no studs
    );
    let tile_class = classifier.classify(&tile_obs);
    print_classification("Yellow Tile", &tile_class);

    println!("\n2. Batch Classification:\n");

    let observations = vec![brick_obs, plate_obs, axle_obs, tile_obs];
    let classifications = classifier.classify_batch(&observations);

    for (i, class) in classifications.iter().enumerate() {
        println!(
            "  Piece {}: {} (confidence: {:.1}%)",
            i + 1,
            class.detected_type.name(),
            class.confidence * 100.0
        );
    }

    println!("\n3. Type-Based Sorting Rules:\n");

    // Define sorting rules
    let rules = vec![
        TypeSortingRule::new(
            "rule_technic".to_string(),
            1, // Highest priority
            TypeSortingConditions::type_only(CoarseType::Axle),
            "bin_technic".to_string(),
        ),
        TypeSortingRule::new(
            "rule_red_plates".to_string(),
            2,
            TypeSortingConditions::type_and_color(CoarseType::Plate, LegoColor::Red),
            "bin_red_plates".to_string(),
        ),
        TypeSortingRule::new(
            "rule_bricks".to_string(),
            3,
            TypeSortingConditions::type_only(CoarseType::Brick),
            "bin_bricks".to_string(),
        ),
        TypeSortingRule::new(
            "rule_plates".to_string(),
            4,
            TypeSortingConditions::type_only(CoarseType::Plate),
            "bin_plates".to_string(),
        ),
    ];

    println!("  Defined Rules:");
    for rule in &rules {
        println!("    - {} (priority: {})", rule.rule_id, rule.priority);
    }

    println!("\n  Applying Rules to Classified Pieces:");

    let test_cases = vec![
        (&brick_class, LegoColor::Red, "Red Brick"),
        (&plate_class, LegoColor::Blue, "Blue Plate"),
        (&axle_class, LegoColor::Black, "Black Axle"),
    ];

    for (class, color, name) in test_cases {
        if let Some(rule) = classifier.find_matching_rule(&rules, class, color) {
            println!(
                "    {} → {} (rule: {})",
                name, rule.target_bin, rule.rule_id
            );
        } else {
            println!("    {} → No matching rule", name);
        }
    }

    println!("\n4. Feature Analysis:\n");

    let features = &brick_class.features;
    println!("  Red Brick Features:");
    println!("    - Has studs: {}", features.has_studs);
    println!("    - Aspect ratio: {:.2}", features.aspect_ratio);
    println!("    - Is flat: {}", features.is_flat);
    println!("    - Is cylindrical: {}", features.is_cylindrical);
    println!("    - Estimated size: {:?}", features.estimated_size);

    println!("\n5. Alternative Classifications:\n");

    if !brick_class.alternative_types.is_empty() {
        println!("  Red Brick alternatives:");
        for alt in &brick_class.alternative_types {
            println!(
                "    - {} (confidence: {:.1}%)",
                alt.piece_type.name(),
                alt.confidence * 100.0
            );
        }
    } else {
        println!("  Red Brick: No alternatives (high confidence in primary classification)");
    }

    println!("\n6. Confidence Thresholding:\n");

    let low_conf_obs = create_observation(
        "ambiguous_001",
        LegoColor::Gray,
        BoundingBox::new(10, 400, 120, 95), // Ambiguous proportions
    );
    let low_conf_class = classifier.classify(&low_conf_obs);

    println!(
        "  Ambiguous piece: {} (confidence: {:.1}%)",
        low_conf_class.detected_type.name(),
        low_conf_class.confidence * 100.0
    );
    println!(
        "    Is reliable (≥70%): {}",
        low_conf_class.is_reliable()
    );

    if !low_conf_class.alternative_types.is_empty() {
        println!("    Consider alternatives:");
        for alt in &low_conf_class.alternative_types {
            println!(
                "      - {} ({:.1}%)",
                alt.piece_type.name(),
                alt.confidence * 100.0
            );
        }
    }

    println!("\n=== Demo Complete ===");
}

fn create_observation(id: &str, color: LegoColor, bbox: BoundingBox) -> PieceObservation {
    let center = bbox.center();
    PieceObservation::new(
        id.to_string(),
        0, // timestamp
        color,
        bbox,
        0.85, // vision confidence
        Position2D::new(center.0 as f32 / 5.0, center.1 as f32 / 5.0),
        PieceSize::Medium,
    )
}

fn print_classification(name: &str, classification: &TypeClassification) {
    println!("  {}:", name);
    println!(
        "    Type: {} (confidence: {:.1}%)",
        classification.detected_type.name(),
        classification.confidence * 100.0
    );
    println!(
        "    Features: aspect_ratio={:.2}, is_flat={}, has_studs={}",
        classification.features.aspect_ratio,
        classification.features.is_flat,
        classification.features.has_studs
    );
    if !classification.alternative_types.is_empty() {
        println!("    Alternatives:");
        for alt in &classification.alternative_types {
            println!(
                "      - {} ({:.1}%)",
                alt.piece_type.name(),
                alt.confidence * 100.0
            );
        }
    }
    println!();
}
