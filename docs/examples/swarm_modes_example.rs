//! Example: Using Swarm Play Modes
//!
//! This example demonstrates how to use the swarm play mode system
//! once the coordination protocol (#82) is integrated.
//!
//! Status: EXAMPLE ONLY - Requires #82 completion

use mbot_core::multi_robot::{
    Position, RobotId, RobotState, RobotRole, RobotStatus,
    swarm::{
        SwarmMode, FollowLeaderMode, CircleMode, WaveMode, RandomWalkMode,
        SwarmConfig, SwarmParams, PlayStatus, FormationType,
    },
    collision::CollisionAvoidance,
};

fn main() {
    println!("=== Swarm Modes Example ===\n");

    // Create mock robots
    let robots = create_test_robots();

    // Example 1: Follow Leader Mode
    example_follow_leader(&robots);

    // Example 2: Circle Formation
    example_circle_formation(&robots);

    // Example 3: Wave Pattern
    example_wave_pattern(&robots);

    // Example 4: Random Walk
    example_random_walk(&robots);

    // Example 5: Collision Avoidance
    example_collision_avoidance(&robots);
}

fn create_test_robots() -> Vec<RobotState> {
    vec![
        create_robot("robot1", 0.0, 0.0, RobotRole::Leader),
        create_robot("robot2", -30.0, 0.0, RobotRole::Follower),
        create_robot("robot3", -60.0, 0.0, RobotRole::Follower),
        create_robot("robot4", -90.0, 0.0, RobotRole::Follower),
    ]
}

fn create_robot(id: &str, x: f32, y: f32, role: RobotRole) -> RobotState {
    let mut robot = RobotState::new(RobotId::new(id.into()));
    robot.position = Position { x, y };
    robot.role = role;
    robot.status = RobotStatus::Idle;
    robot.last_heartbeat = 0;
    robot
}

fn example_follow_leader(robots: &[RobotState]) {
    println!("--- Example 1: Follow Leader ---");

    let mut mode = FollowLeaderMode::new(
        RobotId::new("robot1".into()),
        30.0  // 30cm spacing
    );

    match mode.init(robots) {
        Ok(()) => println!("✓ Follow leader mode initialized"),
        Err(e) => println!("✗ Failed to initialize: {:?}", e),
    }

    // Simulate update
    let delta_time = 0.1;  // 100ms
    match mode.update(delta_time, robots) {
        Ok(targets) => {
            println!("✓ Generated {} target positions", targets.len());
            for target in targets {
                println!(
                    "  {} -> ({:.1}, {:.1}) @ {:.1} cm/s",
                    target.robot_id.as_str(),
                    target.position.x,
                    target.position.y,
                    target.speed
                );
            }
        }
        Err(e) => println!("✗ Update failed: {:?}", e),
    }

    println!();
}

fn example_circle_formation(robots: &[RobotState]) {
    println!("--- Example 2: Circle Formation ---");

    let mut mode = CircleMode::new(
        Position { x: 0.0, y: 0.0 },
        50.0,  // 50cm radius
        0.1    // 0.1 rad/s rotation speed
    );

    match mode.init(robots) {
        Ok(()) => println!("✓ Circle mode initialized"),
        Err(e) => println!("✗ Failed to initialize: {:?}", e),
    }

    // Simulate several updates to show rotation
    for i in 0..5 {
        let delta_time = 1.0;  // 1 second per step
        match mode.update(delta_time, robots) {
            Ok(targets) => {
                println!("✓ Step {} - {} robots in formation", i + 1, targets.len());
            }
            Err(e) => println!("✗ Update failed: {:?}", e),
        }
    }

    println!();
}

fn example_wave_pattern(robots: &[RobotState]) {
    println!("--- Example 3: Wave Pattern ---");

    let mut mode = WaveMode::new(
        20.0,                   // 20cm amplitude
        0.5,                    // 0.5 Hz frequency
        std::f32::consts::PI / 2.0  // 90° phase offset
    );

    match mode.init(robots) {
        Ok(()) => println!("✓ Wave mode initialized"),
        Err(e) => println!("✗ Failed to initialize: {:?}", e),
    }

    // Simulate wave motion
    let delta_time = 0.1;
    match mode.update(delta_time, robots) {
        Ok(targets) => {
            println!("✓ Wave positions generated");
            for (i, target) in targets.iter().enumerate() {
                println!(
                    "  Robot {} at Y = {:.1}cm",
                    i + 1,
                    target.position.y
                );
            }
        }
        Err(e) => println!("✗ Update failed: {:?}", e),
    }

    println!();
}

fn example_random_walk(robots: &[RobotState]) {
    println!("--- Example 4: Random Walk ---");

    let bounds = (
        Position { x: -100.0, y: -100.0 },
        Position { x: 100.0, y: 100.0 }
    );

    let mut mode = RandomWalkMode::new(bounds, 60.0);

    match mode.init(robots) {
        Ok(()) => println!("✓ Random walk mode initialized"),
        Err(e) => println!("✗ Failed to initialize: {:?}", e),
    }

    // Simulate several steps
    for i in 0..3 {
        let delta_time = 5.0;  // 5 seconds per step
        match mode.update(delta_time, robots) {
            Ok(targets) => {
                println!("✓ Step {} - New random positions", i + 1);
                for target in targets {
                    println!(
                        "  {} -> ({:.1}, {:.1})",
                        target.robot_id.as_str(),
                        target.position.x,
                        target.position.y
                    );
                }
            }
            Err(e) => println!("✗ Update failed: {:?}", e),
        }
    }

    println!();
}

fn example_collision_avoidance(robots: &[RobotState]) {
    println!("--- Example 5: Collision Avoidance ---");

    let collision = CollisionAvoidance::new();

    // Test 1: Safe distance
    let target1 = Position { x: 0.0, y: 0.0 };
    let check1 = collision.check_position(&target1, robots);
    println!("Target at (0, 0): Risk = {:?}", check1.risk);

    // Test 2: Too close to a robot
    let target2 = Position { x: -15.0, y: 0.0 };  // 15cm from robot2
    let check2 = collision.check_position(&target2, robots);
    println!("Target at (-15, 0): Risk = {:?}", check2.risk);
    if let Some((avoid_x, avoid_y)) = check2.avoidance_vector {
        println!("  Avoidance vector: ({:.2}, {:.2})", avoid_x, avoid_y);
    }

    // Test 3: Apply avoidance
    let safe_pos = collision.apply_avoidance(&target2, robots);
    println!("  Adjusted position: ({:.1}, {:.1})", safe_pos.x, safe_pos.y);

    // Test 4: Verify swarm safety
    match collision.verify_swarm_safety(robots) {
        Ok(()) => println!("✓ All robots maintain safety buffer"),
        Err(msg) => println!("✗ Safety violation: {}", msg),
    }

    println!();
}

/// Example: Building a full swarm configuration
fn build_swarm_config() -> SwarmConfig {
    SwarmConfig {
        mode_type: mbot_core::multi_robot::swarm::SwarmModeType::Patrol,
        participants: vec![
            RobotId::new("robot1".into()),
            RobotId::new("robot2".into()),
            RobotId::new("robot3".into()),
            RobotId::new("robot4".into()),
        ],
        status: PlayStatus::Idle,
        start_time: 0,
        current_step: 0,
        params: SwarmParams::Patrol {
            formation: FormationType::Diamond,
            spacing: 50.0,  // 50cm between robots
            speed: 20.0,    // 20cm/s movement speed
        },
    }
}

/// Example: Handling robot dropout
fn example_dropout() {
    println!("--- Dropout Handling ---");

    let mut robots = create_test_robots();
    let mut mode = FollowLeaderMode::new(
        RobotId::new("robot1".into()),
        30.0
    );

    mode.init(&robots).unwrap();

    // Simulate robot3 dropping out
    println!("Robot 3 disconnected...");
    let dropout_id = RobotId::new("robot3".into());

    match mode.handle_dropout(&dropout_id) {
        Ok(()) => {
            println!("✓ Mode continues with remaining robots");

            // Remove from robots list
            robots.retain(|r| r.id != dropout_id);

            // Update with remaining robots
            let targets = mode.update(0.1, &robots).unwrap();
            println!("✓ {} robots still in formation", targets.len());
        }
        Err(e) => {
            println!("✗ Mode failed: {:?}", e);
        }
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_runs() {
        // Ensure examples don't panic
        let robots = create_test_robots();
        example_follow_leader(&robots);
        example_circle_formation(&robots);
        example_wave_pattern(&robots);
        example_random_walk(&robots);
        example_collision_avoidance(&robots);
    }

    #[test]
    fn test_build_config() {
        let config = build_swarm_config();
        assert_eq!(config.participants.len(), 4);
        assert_eq!(config.status, PlayStatus::Idle);
    }
}
