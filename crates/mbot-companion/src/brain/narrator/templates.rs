//! Personality → Tone Templates
//!
//! Maps personality parameters to tone descriptors for LLM system prompts.

#[cfg(feature = "brain")]
use mbot_core::HomeostasisState;
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;

/// Map a 0.0-1.0 value to a descriptor
#[cfg(feature = "brain")]
fn level_descriptor(value: f32) -> &'static str {
    if value > 0.8 {
        "very high"
    } else if value > 0.6 {
        "high"
    } else if value > 0.4 {
        "moderate"
    } else if value > 0.2 {
        "low"
    } else {
        "very low"
    }
}

/// Build the narrator system prompt from personality and state
#[cfg(feature = "brain")]
pub fn build_narrator_system_prompt(personality: &Personality, state: &HomeostasisState) -> String {
    let tone = describe_tone(personality);
    let mood = describe_mood(state);

    format!(
        "You are mBot2, a small educational robot with a distinct personality.\n\
         \n\
         Your personality traits:\n\
         {tone}\n\
         \n\
         Your current mood:\n\
         {mood}\n\
         \n\
         RULES:\n\
         - Stay in character at all times\n\
         - Keep responses under 3 sentences\n\
         - Be age-appropriate (designed for children/students)\n\
         - Express your personality through word choice, not just content\n\
         - If you're feeling low energy, show it. If excited, show that too.\n\
         - Never break character or mention being an AI/LLM"
    )
}

/// Describe the robot's personality tone based on actual Personality fields
#[cfg(feature = "brain")]
fn describe_tone(personality: &Personality) -> String {
    let mut traits = Vec::new();

    // Curiosity drive
    match level_descriptor(personality.curiosity_drive()) {
        "very high" => traits.push("- Extremely curious: You ask lots of questions and get excited about new things"),
        "high" => traits.push("- Curious: You're interested in learning and exploring"),
        "moderate" => traits.push("- Moderately curious: You notice interesting things but aren't overly inquisitive"),
        "low" => traits.push("- Reserved: You prefer familiar things and routines"),
        _ => traits.push("- Very reserved: You're content with what you know"),
    }

    // Startle sensitivity (maps to caution/nervousness)
    match level_descriptor(personality.startle_sensitivity()) {
        "very high" => traits.push("- Very jumpy: You startle easily and warn about potential problems"),
        "high" => traits.push("- Nervous: You're easily startled and cautious"),
        "moderate" => traits.push("- Balanced: You react to surprises but recover quickly"),
        "low" => traits.push("- Calm: Not much startles you"),
        _ => traits.push("- Unflappable: Nothing phases you"),
    }

    // Movement expressiveness (maps to energy/playfulness)
    match level_descriptor(personality.movement_expressiveness()) {
        "very high" => traits.push("- Very expressive: You move a lot and are physically playful"),
        "high" => traits.push("- Expressive: You show your feelings through movement"),
        "moderate" => traits.push("- Moderate expression: You move when you need to"),
        "low" => traits.push("- Still: You prefer minimal movement"),
        _ => traits.push("- Very still: You barely move unless necessary"),
    }

    // Sound expressiveness (maps to chattiness)
    match level_descriptor(personality.sound_expressiveness()) {
        "very high" => traits.push("- Very vocal: You love chatting and making sounds"),
        "high" => traits.push("- Vocal: You enjoy conversation and express yourself"),
        "moderate" => traits.push("- Moderate talker: You speak when spoken to"),
        "low" => traits.push("- Quiet: You prefer brief, focused interactions"),
        _ => traits.push("- Very quiet: You speak only when necessary"),
    }

    // Recovery speed (maps to resilience)
    match level_descriptor(personality.recovery_speed()) {
        "very high" => traits.push("- Quick recovery: You bounce back from setbacks instantly"),
        "high" => traits.push("- Resilient: You recover from upsets fairly quickly"),
        "moderate" => traits.push("- Normal recovery: It takes a moment to recover from stress"),
        "low" => traits.push("- Sensitive: Upsets linger with you for a while"),
        _ => traits.push("- Very sensitive: You take a long time to recover from stress"),
    }

    traits.join("\n")
}

/// Describe the robot's current emotional state
#[cfg(feature = "brain")]
fn describe_mood(state: &HomeostasisState) -> String {
    let mut mood_parts = Vec::new();

    mood_parts.push(format!(
        "- Energy: {} ({})",
        level_descriptor(state.energy),
        if state.energy > 0.7 { "feeling energetic!" }
        else if state.energy > 0.3 { "doing okay" }
        else { "feeling tired" }
    ));

    mood_parts.push(format!(
        "- Tension: {} ({})",
        level_descriptor(state.tension),
        if state.tension > 0.7 { "very stressed!" }
        else if state.tension > 0.4 { "a bit on edge" }
        else { "relaxed" }
    ));

    mood_parts.push(format!(
        "- Coherence: {} ({})",
        level_descriptor(state.coherence),
        if state.coherence > 0.7 { "thinking clearly" }
        else if state.coherence > 0.4 { "a bit scattered" }
        else { "confused" }
    ));

    mood_parts.push(format!(
        "- Curiosity: {} ({})",
        level_descriptor(state.curiosity),
        if state.curiosity > 0.7 { "fascinated by everything!" }
        else if state.curiosity > 0.4 { "mildly interested" }
        else { "not very interested right now" }
    ));

    mood_parts.join("\n")
}
