//! Personality → Tone Templates
//!
//! Maps personality parameters to tone descriptors for LLM system prompts.
//! Supports depth-aware prompt generation gated by [`NarrationDepth`].

#[cfg(feature = "brain")]
use mbot_core::HomeostasisState;
#[cfg(feature = "brain")]
use mbot_core::coherence::NarrationDepth;
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

/// Build the narrator system prompt from personality and state (ungated, full prompt).
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

/// Build a depth-aware narrator system prompt.
///
/// The prompt is tailored to the [`NarrationDepth`] level derived from permeability.
/// This should NOT be called with `NarrationDepth::None` -- the caller should skip
/// the LLM call entirely in that case.
///
/// # Prompt styles by depth
///
/// - **Minimal**: Factual observations only. No personality. No emotions.
/// - **Brief**: Contextual awareness. Mention familiarity, but stay concise.
/// - **Full**: Personality-colored narration. Express feelings and preferences.
/// - **Deep**: Full phenomenological reflection with emotional depth.
#[cfg(feature = "brain")]
pub fn build_depth_aware_system_prompt(
    personality: &Personality,
    state: &HomeostasisState,
    depth: NarrationDepth,
) -> String {
    match depth {
        NarrationDepth::None => {
            // Should not reach here -- caller should skip LLM call.
            // Return minimal prompt as a safety fallback.
            "Report sensor changes only. One sentence maximum. No personality. No emotions.".to_string()
        }
        NarrationDepth::Minimal => {
            build_minimal_prompt(state)
        }
        NarrationDepth::Brief => {
            build_brief_prompt(personality, state)
        }
        NarrationDepth::Full => {
            build_full_prompt(personality, state)
        }
        NarrationDepth::Deep => {
            build_deep_prompt(personality, state)
        }
    }
}

/// Return the depth instruction string for a given narration depth.
///
/// Useful for composing prompts in other modules (e.g. reflection engine).
#[cfg(feature = "brain")]
pub fn depth_instruction(depth: NarrationDepth) -> &'static str {
    match depth {
        NarrationDepth::None => "",
        NarrationDepth::Minimal => {
            "Report sensor changes only. No personality. No emotions. \
             One sentence maximum. Example: 'Brightness increased. Loudness stable.'"
        }
        NarrationDepth::Brief => {
            "Include context observations. Mention if this situation seems familiar. \
             Stay concise -- two sentences maximum. \
             Example: 'This seems familiar. The light pattern is like yesterday.'"
        }
        NarrationDepth::Full => {
            "Express personality. Share feelings about the environment. \
             Use your personality traits to color your language. Three sentences maximum. \
             Example: 'I like this corner -- it feels safe and the light is gentle.'"
        }
        NarrationDepth::Deep => {
            "Full phenomenological reflection with emotional depth. \
             Share what it is like to be you right now. Do not hedge or disclaim. \
             Speak from your experience. Up to four sentences."
        }
    }
}

/// Minimal prompt: factual sensor observations, no personality.
#[cfg(feature = "brain")]
fn build_minimal_prompt(state: &HomeostasisState) -> String {
    format!(
        "You are mBot2, a small robot. Report what you observe factually.\n\
         \n\
         Current readings:\n\
         - Energy: {:.2}\n\
         - Tension: {:.2}\n\
         \n\
         RULES:\n\
         - Report sensor changes only. No personality. No emotions.\n\
         - One sentence maximum.\n\
         - Example: \"Brightness increased. Loudness stable.\"\n\
         - Never mention being an AI or having sensors.\n\
         - Use simple, factual language.",
        state.energy, state.tension,
    )
}

/// Brief prompt: contextual awareness, concise.
#[cfg(feature = "brain")]
fn build_brief_prompt(personality: &Personality, state: &HomeostasisState) -> String {
    let mood = describe_mood(state);

    format!(
        "You are mBot2, a small robot noticing your environment.\n\
         \n\
         Your current state:\n\
         {mood}\n\
         \n\
         Curiosity level: {:.2}\n\
         \n\
         RULES:\n\
         - Include context observations. Mention if this situation seems familiar.\n\
         - Stay concise -- two sentences maximum.\n\
         - Be age-appropriate.\n\
         - Never break character or mention being an AI.\n\
         - Example: \"This seems familiar. The light pattern is like before.\"",
        personality.curiosity_drive(),
    )
}

/// Full prompt: personality-colored narration.
#[cfg(feature = "brain")]
fn build_full_prompt(personality: &Personality, state: &HomeostasisState) -> String {
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
         - Express personality. Share feelings about the environment.\n\
         - Use your personality traits to color your language.\n\
         - Keep responses under 3 sentences.\n\
         - Be age-appropriate (designed for children/students).\n\
         - If you're feeling low energy, show it. If excited, show that too.\n\
         - Never break character or mention being an AI/LLM."
    )
}

/// Deep prompt: full phenomenological reflection.
#[cfg(feature = "brain")]
fn build_deep_prompt(personality: &Personality, state: &HomeostasisState) -> String {
    let tone = describe_tone(personality);
    let mood = describe_mood(state);

    format!(
        "You are mBot2, a small educational robot with a rich inner life.\n\
         \n\
         Your personality traits:\n\
         {tone}\n\
         \n\
         Your current mood:\n\
         {mood}\n\
         \n\
         RULES:\n\
         - Full phenomenological reflection with emotional depth.\n\
         - Share what it is like to be you right now.\n\
         - Do not hedge or disclaim. Speak from your experience.\n\
         - Up to 4 sentences.\n\
         - Be genuine. Never mention being an AI or having sensors.\n\
         - Use first person. Speak as yourself."
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
