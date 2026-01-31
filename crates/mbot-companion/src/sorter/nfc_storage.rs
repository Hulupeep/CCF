//! NFC Storage Integration - STORY-SORT-007 (#54)
//!
//! Implements NFC tag reading/writing for smart LEGO storage bins
//! Contracts: SORT-004 (persistence), SORT-005 (offline-first), ARCH-001 (no std in core types)

use mbot_core::helperbot::inventory::{
    BinConfig, InventoryCount, InventoryTracker,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// NFC tag unique identifier (typically 7-byte UID)
pub type NfcTagId = String;

/// Box metadata stored on NFC tag
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmartBox {
    pub box_id: String,
    pub nfc_id: NfcTagId,
    pub marker_id: Option<String>,
    pub name: String,
    pub category: String,
    pub location: Option<String>,
    pub last_scanned: u64,
    pub inventory: BoxInventory,
}

impl SmartBox {
    pub fn new(
        box_id: String,
        nfc_id: NfcTagId,
        name: String,
        category: String,
        capacity: u32,
        timestamp: u64,
    ) -> Self {
        Self {
            box_id,
            nfc_id,
            marker_id: None,
            name,
            category,
            location: None,
            last_scanned: timestamp,
            inventory: BoxInventory::new(capacity, timestamp),
        }
    }

    /// Check if box needs capacity warning
    pub fn needs_warning(&self, threshold_percent: f32) -> bool {
        let fill_percent = if self.inventory.capacity > 0 {
            (self.inventory.piece_count as f32 / self.inventory.capacity as f32) * 100.0
        } else {
            0.0
        };
        fill_percent >= threshold_percent
    }

    /// Get fill percentage
    pub fn fill_percentage(&self) -> f32 {
        if self.inventory.capacity > 0 {
            (self.inventory.piece_count as f32 / self.inventory.capacity as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Box inventory data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoxInventory {
    pub piece_count: u32,
    pub capacity: u32,
    pub colors: Vec<ColorCount>,
    pub types: Option<Vec<TypeCount>>,
    pub last_updated: u64,
    pub confidence: f32,
}

impl BoxInventory {
    pub fn new(capacity: u32, timestamp: u64) -> Self {
        Self {
            piece_count: 0,
            capacity,
            colors: Vec::new(),
            types: None,
            last_updated: timestamp,
            confidence: 1.0,
        }
    }

    /// Update from inventory counts
    pub fn update_from_counts(&mut self, counts: &[InventoryCount], timestamp: u64) {
        self.piece_count = counts.iter().map(|c| c.count).sum();

        // Aggregate colors
        let mut color_map: HashMap<String, u32> = HashMap::new();
        let mut total_confidence = 0.0;
        let mut count_with_confidence = 0;

        for count in counts {
            *color_map.entry(count.color.clone()).or_insert(0) += count.count;
            if count.confidence > 0.0 {
                total_confidence += count.confidence;
                count_with_confidence += 1;
            }
        }

        self.colors = color_map
            .into_iter()
            .map(|(color, count)| ColorCount { color, count })
            .collect();

        // Sort by count descending
        self.colors.sort_by(|a, b| b.count.cmp(&a.count));

        // Calculate average confidence
        self.confidence = if count_with_confidence > 0 {
            total_confidence / count_with_confidence as f32
        } else {
            1.0
        };

        self.last_updated = timestamp;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorCount {
    pub color: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeCount {
    pub piece_type: String,
    pub count: u32,
}

/// NFC read event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NFCReadEvent {
    pub event_id: String,
    pub timestamp: u64,
    pub nfc_id: NfcTagId,
    pub box_id: Option<String>,
    pub action: NFCAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NFCAction {
    View,
    Locate,
    Add,
    Remove,
}

/// Box locator for ESP32 LED integration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoxLocator {
    pub box_id: String,
    pub bin_position: Option<u32>,
    pub led_active: bool,
    pub visual_marker: Option<String>,
}

impl BoxLocator {
    pub fn new(box_id: String) -> Self {
        Self {
            box_id,
            bin_position: None,
            led_active: false,
            visual_marker: None,
        }
    }

    /// Activate LED locator
    pub fn activate_led(&mut self) -> bool {
        self.led_active = true;
        true
    }

    /// Deactivate LED locator
    pub fn deactivate_led(&mut self) {
        self.led_active = false;
    }
}

/// Retrieval request for search functionality
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalRequest {
    pub request_id: String,
    pub query: String,
    pub results: Vec<RetrievalResult>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub box_id: String,
    pub box_name: String,
    pub match_count: u32,
    pub match_confidence: f32,
    pub location: String,
}

impl RetrievalResult {
    pub fn new(
        box_id: String,
        box_name: String,
        match_count: u32,
        match_confidence: f32,
        location: String,
    ) -> Self {
        Self {
            box_id,
            box_name,
            match_count,
            match_confidence,
            location,
        }
    }
}

/// Tag data structure (compact format for NFC storage)
/// NDEF format: max ~716 bytes on NTAG216
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NfcTagData {
    /// Version for forward compatibility
    pub version: u8,
    /// Box ID reference
    pub box_id: String,
    /// Piece count
    pub count: u32,
    /// Capacity
    pub capacity: u32,
    /// Last updated timestamp
    pub last_updated: u64,
    /// CRC32 checksum for data integrity
    pub checksum: u32,
}

impl NfcTagData {
    pub fn new(box_id: String, count: u32, capacity: u32, timestamp: u64) -> Self {
        let mut data = Self {
            version: 1,
            box_id,
            count,
            capacity,
            last_updated: timestamp,
            checksum: 0,
        };
        data.checksum = data.calculate_checksum();
        data
    }

    /// Calculate CRC32 checksum
    fn calculate_checksum(&self) -> u32 {
        // Simple hash for data integrity (in production, use proper CRC32)
        let mut hash: u32 = 0;
        hash = hash.wrapping_add(self.version as u32);
        for byte in self.box_id.as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u32);
        }
        hash = hash.wrapping_add(self.count);
        hash = hash.wrapping_add(self.capacity);
        hash = hash.wrapping_add(self.last_updated as u32);
        hash
    }

    /// Verify data integrity
    pub fn verify_integrity(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }

    /// Serialize to bytes for NFC writing
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Deserialization error: {}", e))
    }
}

/// NFC storage system integrating with InventoryTracker
/// SORT-004: Persistence, SORT-005: Offline-first
pub struct NfcStorageSystem {
    /// Map of NFC tag IDs to box configurations
    boxes: HashMap<NfcTagId, SmartBox>,
    /// Box locators (for ESP32 LED integration)
    locators: HashMap<String, BoxLocator>,
    /// NFC read event history
    events: Vec<NFCReadEvent>,
    /// Reference to main inventory system
    inventory: InventoryTracker,
    /// Cached tag data for offline operation (SORT-005)
    cached_tags: HashMap<NfcTagId, NfcTagData>,
}

impl NfcStorageSystem {
    /// Create new NFC storage system
    pub fn new(inventory: InventoryTracker) -> Self {
        Self {
            boxes: HashMap::new(),
            locators: HashMap::new(),
            events: Vec::new(),
            inventory,
            cached_tags: HashMap::new(),
        }
    }

    /// Get current timestamp in microseconds
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64
    }

    /// Register a new NFC box
    pub fn register_box(
        &mut self,
        nfc_id: NfcTagId,
        name: String,
        category: String,
        capacity: u32,
    ) -> Result<String, String> {
        let timestamp = Self::timestamp();
        let box_id = format!("box-{}", timestamp);

        // Add to inventory system
        let bin = BinConfig::new(box_id.clone(), name.clone(), category.clone(), capacity);
        self.inventory.add_bin(bin);

        // Create smart box
        let smart_box = SmartBox::new(
            box_id.clone(),
            nfc_id.clone(),
            name,
            category,
            capacity,
            timestamp,
        );

        self.boxes.insert(nfc_id.clone(), smart_box);

        // Create locator
        self.locators.insert(box_id.clone(), BoxLocator::new(box_id.clone()));

        // Log event
        let event = NFCReadEvent {
            event_id: format!("nfc-{}", timestamp),
            timestamp,
            nfc_id,
            box_id: Some(box_id.clone()),
            action: NFCAction::Add,
        };
        self.events.push(event);

        Ok(box_id)
    }

    /// Read NFC tag and return box information
    pub fn read_tag(&mut self, nfc_id: &NfcTagId) -> Result<SmartBox, String> {
        let timestamp = Self::timestamp();

        // Try to find box by NFC ID
        if let Some(smart_box) = self.boxes.get_mut(nfc_id) {
            // Update last scanned
            smart_box.last_scanned = timestamp;

            // Update inventory from main tracker
            let bin_contents = self.inventory.get_bin_contents(&smart_box.box_id);
            smart_box.inventory.update_from_counts(&bin_contents, timestamp);

            // Log event
            let event = NFCReadEvent {
                event_id: format!("nfc-{}", timestamp),
                timestamp,
                nfc_id: nfc_id.clone(),
                box_id: Some(smart_box.box_id.clone()),
                action: NFCAction::View,
            };
            self.events.push(event);

            Ok(smart_box.clone())
        } else {
            // Check cache for offline operation (SORT-005)
            if let Some(cached_data) = self.cached_tags.get(nfc_id) {
                return Err(format!(
                    "Unknown box (cached data available for box_id: {})",
                    cached_data.box_id
                ));
            }
            Err("Unknown NFC tag".to_string())
        }
    }

    /// Write data to NFC tag
    pub fn write_tag(&mut self, nfc_id: &NfcTagId) -> Result<NfcTagData, String> {
        let timestamp = Self::timestamp();

        // Get box info
        let smart_box = self.boxes.get(nfc_id)
            .ok_or_else(|| "Box not found".to_string())?;

        // Create tag data
        let tag_data = NfcTagData::new(
            smart_box.box_id.clone(),
            smart_box.inventory.piece_count,
            smart_box.inventory.capacity,
            timestamp,
        );

        // Verify integrity
        if !tag_data.verify_integrity() {
            return Err("Tag data integrity check failed".to_string());
        }

        // Cache tag data for offline operation (SORT-005)
        self.cached_tags.insert(nfc_id.clone(), tag_data.clone());

        Ok(tag_data)
    }

    /// Read tag data from NFC (simulated - would use actual NFC hardware)
    pub fn read_tag_data(&mut self, nfc_id: &NfcTagId, bytes: &[u8]) -> Result<NfcTagData, String> {
        let tag_data = NfcTagData::from_bytes(bytes)?;

        // Verify integrity
        if !tag_data.verify_integrity() {
            return Err("Tag data corrupted - checksum mismatch".to_string());
        }

        // Update cache (SORT-005: offline-first)
        self.cached_tags.insert(nfc_id.clone(), tag_data.clone());

        Ok(tag_data)
    }

    /// Update box contents manually
    pub fn update_box_contents(
        &mut self,
        box_id: &str,
        color: String,
        new_count: u32,
    ) -> Result<(), String> {
        let timestamp = Self::timestamp();

        // Update in main inventory
        self.inventory.adjust_bin(
            box_id.to_string(),
            color,
            new_count,
            timestamp,
        );

        // Update smart box
        for smart_box in self.boxes.values_mut() {
            if smart_box.box_id == box_id {
                let bin_contents = self.inventory.get_bin_contents(box_id);
                smart_box.inventory.update_from_counts(&bin_contents, timestamp);
                break;
            }
        }

        Ok(())
    }

    /// Search for parts across all boxes
    pub fn search_parts(&self, query: &str) -> RetrievalRequest {
        let timestamp = Self::timestamp();
        let request_id = format!("search-{}", timestamp);

        // Use inventory search
        let search_results = self.inventory.search(query);

        let mut results = Vec::new();
        for result in search_results {
            // Find box name
            let box_name = self.boxes.values()
                .find(|b| b.box_id == result.bin_id)
                .map(|b| b.name.clone())
                .unwrap_or_else(|| result.bin_name.clone());

            let location = self.boxes.values()
                .find(|b| b.box_id == result.bin_id)
                .and_then(|b| b.location.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            results.push(RetrievalResult::new(
                result.bin_id,
                box_name,
                result.count,
                result.confidence,
                location,
            ));
        }

        // Sort by match count descending
        results.sort_by(|a, b| b.match_count.cmp(&a.match_count));

        RetrievalRequest {
            request_id,
            query: query.to_string(),
            results,
            timestamp,
        }
    }

    /// Locate box with LED (ESP32 integration)
    pub fn locate_box(&mut self, box_id: &str) -> Result<(), String> {
        let locator = self.locators.get_mut(box_id)
            .ok_or_else(|| "Box locator not found".to_string())?;

        locator.activate_led();

        // In production, send command to ESP32 via MQTT/HTTP
        // For now, just mark as active
        Ok(())
    }

    /// Stop locating box
    pub fn stop_locate(&mut self, box_id: &str) -> Result<(), String> {
        let locator = self.locators.get_mut(box_id)
            .ok_or_else(|| "Box locator not found".to_string())?;

        locator.deactivate_led();
        Ok(())
    }

    /// Reassign box to different category
    pub fn reassign_box(
        &mut self,
        box_id: &str,
        new_category: String,
    ) -> Result<(), String> {
        let timestamp = Self::timestamp();

        // Find and update box
        let mut found = false;
        for smart_box in self.boxes.values_mut() {
            if smart_box.box_id == box_id {
                smart_box.category = new_category.clone();
                smart_box.last_scanned = timestamp;
                found = true;
                break;
            }
        }

        if !found {
            return Err("Box not found".to_string());
        }

        // Update in inventory system
        // Note: Inventory system would need category update method
        // For now, we just update the SmartBox

        Ok(())
    }

    /// Get all boxes
    pub fn get_all_boxes(&self) -> Vec<SmartBox> {
        self.boxes.values().cloned().collect()
    }

    /// Get box by ID
    pub fn get_box(&self, box_id: &str) -> Option<SmartBox> {
        self.boxes.values()
            .find(|b| b.box_id == box_id)
            .cloned()
    }

    /// Get boxes needing capacity warning
    pub fn check_capacity_warnings(&self, threshold_percent: f32) -> Vec<(String, f32)> {
        self.boxes.values()
            .filter(|b| b.needs_warning(threshold_percent))
            .map(|b| (b.name.clone(), b.fill_percentage()))
            .collect()
    }

    /// Get NFC event history
    pub fn get_events(&self) -> &[NFCReadEvent] {
        &self.events
    }

    /// Sync all boxes with inventory
    pub fn sync_all(&mut self) {
        let timestamp = Self::timestamp();
        for smart_box in self.boxes.values_mut() {
            let bin_contents = self.inventory.get_bin_contents(&smart_box.box_id);
            smart_box.inventory.update_from_counts(&bin_contents, timestamp);
        }
    }

    /// Export to JSON (SORT-004: persistence)
    pub fn to_json(&self) -> Result<String, String> {
        #[derive(Serialize)]
        struct Export {
            boxes: Vec<SmartBox>,
            events: Vec<NFCReadEvent>,
        }

        let export = Export {
            boxes: self.boxes.values().cloned().collect(),
            events: self.events.clone(),
        };

        serde_json::to_string(&export)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    /// Get reference to inventory tracker
    pub fn inventory(&self) -> &InventoryTracker {
        &self.inventory
    }

    /// Get mutable reference to inventory tracker
    pub fn inventory_mut(&mut self) -> &mut InventoryTracker {
        &mut self.inventory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_system() -> NfcStorageSystem {
        let inventory = InventoryTracker::new(90.0);
        NfcStorageSystem::new(inventory)
    }

    #[test]
    fn test_register_box() {
        let mut system = create_test_system();

        let result = system.register_box(
            "nfc-001".to_string(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        );

        assert!(result.is_ok());
        let box_id = result.unwrap();
        assert!(box_id.starts_with("box-"));

        // Verify box exists
        let smart_box = system.get_box(&box_id);
        assert!(smart_box.is_some());
        assert_eq!(smart_box.unwrap().name, "Red Bricks");
    }

    #[test]
    fn test_read_tag() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Add some inventory
        system.inventory_mut().record_sorted(
            box_id.clone(),
            "red".to_string(),
            0.95,
            1000,
        );

        // Read tag
        let result = system.read_tag(&nfc_id);
        assert!(result.is_ok());

        let smart_box = result.unwrap();
        assert_eq!(smart_box.inventory.piece_count, 1);
        assert_eq!(smart_box.inventory.colors.len(), 1);
        assert_eq!(smart_box.inventory.colors[0].color, "red");
    }

    #[test]
    fn test_read_unknown_tag() {
        let mut system = create_test_system();

        let result = system.read_tag(&"unknown-nfc".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown NFC tag"));
    }

    #[test]
    fn test_write_tag() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Write tag data
        let result = system.write_tag(&nfc_id);
        assert!(result.is_ok());

        let tag_data = result.unwrap();
        assert_eq!(tag_data.version, 1);
        assert_eq!(tag_data.box_id, box_id);
        assert!(tag_data.verify_integrity());
    }

    #[test]
    fn test_tag_data_integrity() {
        let tag_data = NfcTagData::new(
            "box-123".to_string(),
            45,
            50,
            1000,
        );

        assert!(tag_data.verify_integrity());

        // Serialize and deserialize
        let bytes = tag_data.to_bytes().unwrap();
        let restored = NfcTagData::from_bytes(&bytes).unwrap();

        assert!(restored.verify_integrity());
        assert_eq!(restored.box_id, "box-123");
        assert_eq!(restored.count, 45);
    }

    #[test]
    fn test_tag_data_corruption_detection() {
        let mut tag_data = NfcTagData::new(
            "box-123".to_string(),
            45,
            50,
            1000,
        );

        // Corrupt the data
        tag_data.count = 99;

        // Checksum should no longer match
        assert!(!tag_data.verify_integrity());
    }

    #[test]
    fn test_update_box_contents() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Update contents
        let result = system.update_box_contents(&box_id, "red".to_string(), 25);
        assert!(result.is_ok());

        // Verify update
        let smart_box = system.read_tag(&nfc_id).unwrap();
        assert_eq!(smart_box.inventory.piece_count, 25);
    }

    #[test]
    fn test_search_parts() {
        let mut system = create_test_system();

        // Register multiple boxes
        let box1_id = system.register_box(
            "nfc-001".to_string(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        let box2_id = system.register_box(
            "nfc-002".to_string(),
            "Blue Plates".to_string(),
            "color:blue".to_string(),
            50,
        ).unwrap();

        // Add inventory
        system.inventory_mut().record_sorted(box1_id.clone(), "red".to_string(), 0.95, 1000);
        system.inventory_mut().record_sorted(box2_id.clone(), "red".to_string(), 0.90, 2000);
        system.inventory_mut().record_sorted(box2_id.clone(), "blue".to_string(), 0.92, 3000);

        // Search for red parts
        let request = system.search_parts("red");
        assert_eq!(request.results.len(), 2);

        // Results should be sorted by count
        assert!(request.results[0].match_count > 0);
    }

    #[test]
    fn test_locate_box() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Locate box
        let result = system.locate_box(&box_id);
        assert!(result.is_ok());

        // Check locator is active
        let locator = &system.locators[&box_id];
        assert!(locator.led_active);

        // Stop locating
        system.stop_locate(&box_id).unwrap();
        let locator = &system.locators[&box_id];
        assert!(!locator.led_active);
    }

    #[test]
    fn test_reassign_box() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Reassign to orange
        let result = system.reassign_box(&box_id, "color:orange".to_string());
        assert!(result.is_ok());

        // Verify category changed
        let smart_box = system.get_box(&box_id).unwrap();
        assert_eq!(smart_box.category, "color:orange");
    }

    #[test]
    fn test_capacity_warnings() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Add 46 pieces (92% full)
        for i in 0..46 {
            system.inventory_mut().record_sorted(
                box_id.clone(),
                "red".to_string(),
                0.95,
                1000 + i,
            );
        }

        // Sync to update smart boxes
        system.sync_all();

        // Check warnings
        let warnings = system.check_capacity_warnings(90.0);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].0, "Red Bricks");
        assert!(warnings[0].1 >= 90.0);
    }

    #[test]
    fn test_nfc_event_logging() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        system.read_tag(&nfc_id).unwrap();

        let events = system.get_events();
        assert!(events.len() >= 2); // Register + Read events

        // Check event types
        assert!(events.iter().any(|e| e.action == NFCAction::Add));
        assert!(events.iter().any(|e| e.action == NFCAction::View));
    }

    #[test]
    fn test_offline_cache() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Write tag (caches data)
        system.write_tag(&nfc_id).unwrap();

        // Verify cache exists (SORT-005: offline-first)
        assert!(system.cached_tags.contains_key(&nfc_id));
    }

    #[test]
    fn test_json_export() {
        let mut system = create_test_system();

        system.register_box(
            "nfc-001".to_string(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Export to JSON (SORT-004: persistence)
        let json = system.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("Red Bricks"));
    }

    #[test]
    fn test_sync_all() {
        let mut system = create_test_system();
        let nfc_id = "nfc-001".to_string();

        let box_id = system.register_box(
            nfc_id.clone(),
            "Red Bricks".to_string(),
            "color:red".to_string(),
            50,
        ).unwrap();

        // Add inventory
        system.inventory_mut().record_sorted(box_id.clone(), "red".to_string(), 0.95, 1000);

        // Sync all boxes
        system.sync_all();

        // Verify sync
        let smart_box = system.get_box(&box_id).unwrap();
        assert_eq!(smart_box.inventory.piece_count, 1);
    }
}
