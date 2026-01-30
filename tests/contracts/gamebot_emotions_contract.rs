//! Contract tests for GameBot Emotional Responses
//!
//! Enforces GAME-001..007 and I-GAME-004..006 contracts
//! from docs/contracts/feature_gamebot.yml

use mbot_core::gamebot::emotions::*;

#[test]
fn test_emotion_behavior_thinking() {
    // GAME-003: Thinking behavior shows visible LED pattern
    let behavior = EmotionBehavior::thinking(2000, 0.5);

    assert_eq!(behavior.movement, MovementType::Wiggle);
    assert_eq!(behavior.sound, Some(EmotionSound::Hum));
    assert_eq!(behavior.led.pattern, LedPattern::Pulse);
    assert_eq!(behavior.led.primary_color, [0, 100, 200]); // Blue
}

#[test]
fn test_emotion_behavior_thinking_personality_scaling() {
    // GAME-003 @personality: Nervous personality thinks longer
    let anxious = EmotionBehavior::thinking(2000, 0.9);
    let confident = EmotionBehavior::thinking(2000, 0.1);

    // Anxious should have longer thinking time and faster animation
    assert!(anxious.duration_ms > confident.duration_ms);
    assert_eq!(anxious.led.speed, AnimationSpeed::Fast);
    assert_eq!(confident.led.speed, AnimationSpeed::Slow);
}

#[test]
fn test_emotion_behavior_victory() {
    // GAME-003 @victory: Robot celebrates winning
    let behavior = EmotionBehavior::victory(0.7, 0.5);

    assert_eq!(behavior.movement, MovementType::Spin);
    assert_eq!(behavior.sound, Some(EmotionSound::Celebration));
    assert_eq!(behavior.led.pattern, LedPattern::Flash);
    assert_eq!(behavior.led.primary_color, [0, 255, 0]); // Green
}

#[test]
fn test_emotion_behavior_victory_closenesss_scaling() {
    // GAME-003 @closeness: Close victory triggers stronger celebration
    let blowout = EmotionBehavior::victory(0.5, 0.1);
    let close = EmotionBehavior::victory(0.5, 0.9);

    // Close game should have longer celebration
    assert!(close.duration_ms >= blowout.duration_ms);
}

#[test]
fn test_emotion_behavior_loss_non_aggressive() {
    // I-GAME-004: Loss must NEVER be aggressive
    let behavior = EmotionBehavior::loss();

    // Should NOT be red (aggression)
    assert_ne!(behavior.led.primary_color, [255, 0, 0]);

    // Should be orange/sad, not aggressive
    assert_eq!(behavior.movement, MovementType::Slump);
    assert_eq!(behavior.sound, Some(EmotionSound::Sad));
    assert_eq!(behavior.led.primary_color, [255, 165, 0]); // Orange
}

#[test]
fn test_emotion_behavior_draw() {
    // GAME-003 @draw: Robot responds to draw appropriately
    let behavior = EmotionBehavior::draw();

    assert_eq!(behavior.movement, MovementType::Shrug);
    assert_eq!(behavior.sound, Some(EmotionSound::Beep));
    assert_eq!(behavior.led.pattern, LedPattern::Solid);
    assert_eq!(behavior.led.primary_color, [255, 255, 0]); // Yellow
}

#[test]
fn test_game_emotion_context_rematch_always_offered() {
    // I-GAME-006: Robot must ALWAYS offer to play again after any game outcome
    let won = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5);
    let lost = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Lost, 0.5);
    let draw = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Draw, 0.5);

    assert!(won.should_offer_rematch());
    assert!(lost.should_offer_rematch());
    assert!(draw.should_offer_rematch());

    // But NOT during thinking
    let thinking = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Thinking, 0.5);
    assert!(!thinking.should_offer_rematch());

    // Rematch prompt should always be the same
    assert_eq!(won.rematch_prompt(), "Play again?");
    assert_eq!(lost.rematch_prompt(), "Play again?");
}

#[test]
fn test_game_emotion_context_generate_behavior_victory() {
    let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.8)
        .with_closeness(0.9);

    let behavior = context.generate_behavior();

    assert_eq!(behavior.movement, MovementType::Spin);
    assert_eq!(behavior.sound, Some(EmotionSound::Celebration));
}

#[test]
fn test_game_emotion_context_generate_behavior_loss_non_aggressive() {
    // I-GAME-004: Loss must never be aggressive
    let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Lost, 0.9);
    let behavior = context.generate_behavior();

    assert_eq!(behavior.movement, MovementType::Slump);
    assert_ne!(behavior.led.primary_color, [255, 0, 0]); // NOT red
}

#[test]
fn test_game_emotion_context_intensity_clamping() {
    // Intensity should be clamped to [0.0, 1.0]
    let over = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 1.5);
    let under = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, -0.5);

    assert_eq!(over.intensity, 1.0);
    assert_eq!(under.intensity, 0.0);
}

#[test]
fn test_game_emotion_context_closeness_clamping() {
    let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5)
        .with_closeness(1.5); // Invalid

    assert_eq!(context.game_closeness, 1.0);

    let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5)
        .with_closeness(-0.5); // Invalid

    assert_eq!(context.game_closeness, 0.0);
}

#[test]
fn test_game_emotion_context_streak_tracking() {
    let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5)
        .with_streak(3);

    assert_eq!(context.streak, 3);
}

#[test]
fn test_led_spec_presets() {
    let thinking = LedSpec::thinking();
    assert_eq!(thinking.pattern, LedPattern::Pulse);
    assert_eq!(thinking.primary_color, [0, 100, 200]);

    let victory = LedSpec::victory();
    assert_eq!(victory.pattern, LedPattern::Flash);
    assert_eq!(victory.primary_color, [0, 255, 0]);

    let loss = LedSpec::loss();
    assert_eq!(loss.pattern, LedPattern::Pulse);
    assert_eq!(loss.primary_color, [255, 165, 0]);

    let draw = LedSpec::draw();
    assert_eq!(draw.pattern, LedPattern::Solid);
    assert_eq!(draw.primary_color, [255, 255, 0]);
}

#[test]
fn test_emotion_behavior_with_sound() {
    let behavior = EmotionBehavior::new(
        LedSpec::victory(),
        MovementType::Bounce,
        3000,
    ).with_sound(EmotionSound::Celebration);

    assert_eq!(behavior.sound, Some(EmotionSound::Celebration));
}

#[test]
fn test_emotion_behavior_with_repeat() {
    let behavior = EmotionBehavior::new(
        LedSpec::draw(),
        MovementType::Shrug,
        1500,
    ).with_repeat(0); // Zero should become 1

    assert_eq!(behavior.repeat_count, 1);

    let behavior = EmotionBehavior::new(
        LedSpec::draw(),
        MovementType::Shrug,
        1500,
    ).with_repeat(3);

    assert_eq!(behavior.repeat_count, 3);
}

#[test]
fn test_game_types_exist() {
    // Verify all game types can be created
    let _tictactoe = GameType::TicTacToe;
    let _chase = GameType::Chase;
    let _simon = GameType::Simon;
    let _dance = GameType::Dance;
    let _hide_seek = GameType::HideSeek;
}

#[test]
fn test_multiple_game_outcomes() {
    // Verify all game outcomes generate appropriate behaviors
    let thinking = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Thinking, 0.5);
    let thinking_behavior = thinking.generate_behavior();
    assert_eq!(thinking_behavior.movement, MovementType::Wiggle);

    let won = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5);
    let won_behavior = won.generate_behavior();
    assert_eq!(won_behavior.movement, MovementType::Spin);

    let lost = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Lost, 0.5);
    let lost_behavior = lost.generate_behavior();
    assert_eq!(lost_behavior.movement, MovementType::Slump);

    let draw = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Draw, 0.5);
    let draw_behavior = draw.generate_behavior();
    assert_eq!(draw_behavior.movement, MovementType::Shrug);
}

#[test]
fn test_animation_speeds() {
    // Verify animation speeds are distinct
    assert_ne!(AnimationSpeed::Slow, AnimationSpeed::Medium);
    assert_ne!(AnimationSpeed::Medium, AnimationSpeed::Fast);
    assert_ne!(AnimationSpeed::Slow, AnimationSpeed::Fast);
}

#[test]
fn test_led_colors_distinct() {
    // Different emotions should have distinct colors (for clarity)
    let thinking = LedSpec::thinking().primary_color;
    let victory = LedSpec::victory().primary_color;
    let loss = LedSpec::loss().primary_color;
    let draw = LedSpec::draw().primary_color;

    assert_ne!(thinking, victory);
    assert_ne!(victory, loss);
    assert_ne!(loss, draw);
    assert_ne!(thinking, draw);
}
