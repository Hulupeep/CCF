//! Downward messages from companion (deliberative) to core (reflexive).
//!
//! These messages carry learned parameters back down to the core's
//! deterministic nervous system for modulation.
//!
//! Lives in mbot-companion (std) because it uses Vec for variable-size data.
//! The core consumes these messages through the main loop integration point.

use mbot_core::nervous_system::suppression::SuppressionMap;

/// Messages the companion sends to the core for modulation.
#[derive(Clone, Debug)]
pub enum DownwardMessage {
    /// Updated suppression map from the learning layer.
    SuppressionMapUpdate(SuppressionMap),
    /// Updated coherence group map (context_hash -> group_id).
    CoherenceGroupMap(Vec<(u32, u32)>),
    /// Signal that consolidation is starting (true) or ending (false).
    ConsolidationState(bool),
}

#[cfg(test)]
mod tests {
    use super::*;
    use mbot_core::nervous_system::stimulus::StimulusKind;
    use mbot_core::nervous_system::suppression::{SuppressionMap, SuppressionRule};

    #[test]
    fn suppression_map_update_holds_map() {
        let mut map = SuppressionMap::new();
        map.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::LoudnessSpike,
            context_hash: 42,
            suppression_factor: 0.5,
            observation_count: 10,
            last_updated_tick: 100,
        });

        let msg = DownwardMessage::SuppressionMapUpdate(map);

        // Verify we can match and access the inner map
        match msg {
            DownwardMessage::SuppressionMapUpdate(m) => {
                assert_eq!(m.lookup(StimulusKind::LoudnessSpike, 42), 0.5);
                assert_eq!(m.len(), 1);
            }
            _ => panic!("expected SuppressionMapUpdate"),
        }
    }

    #[test]
    fn coherence_group_map_holds_pairs() {
        let pairs = vec![(100, 1), (200, 1), (300, 2)];
        let msg = DownwardMessage::CoherenceGroupMap(pairs);

        match msg {
            DownwardMessage::CoherenceGroupMap(groups) => {
                assert_eq!(groups.len(), 3);
                assert_eq!(groups[0], (100, 1));
                assert_eq!(groups[2], (300, 2));
            }
            _ => panic!("expected CoherenceGroupMap"),
        }
    }

    #[test]
    fn consolidation_state_true_and_false() {
        let start = DownwardMessage::ConsolidationState(true);
        let end = DownwardMessage::ConsolidationState(false);

        match start {
            DownwardMessage::ConsolidationState(active) => assert!(active),
            _ => panic!("expected ConsolidationState"),
        }
        match end {
            DownwardMessage::ConsolidationState(active) => assert!(!active),
            _ => panic!("expected ConsolidationState"),
        }
    }

    #[test]
    fn downward_message_is_clone() {
        let msg = DownwardMessage::ConsolidationState(true);
        let cloned = msg.clone();
        match cloned {
            DownwardMessage::ConsolidationState(active) => assert!(active),
            _ => panic!("expected ConsolidationState"),
        }
    }
}
