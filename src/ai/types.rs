use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct IdentifyResult {
    pub common_name: String,
    pub scientific_name: String,
    pub confidence: Option<f64>,
    pub summary: Option<String>,
    pub care_profile: Option<CareProfile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CareProfile {
    pub watering_interval_days: Option<u32>,
    pub light_needs: Option<String>,
    pub difficulty: Option<String>,
    pub pet_safety: Option<String>,
    pub growth_speed: Option<String>,
    pub soil_type: Option<String>,
    pub soil_moisture: Option<String>,
}

pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub type ChatResponseStream = tokio_stream::wrappers::ReceiverStream<Result<String, String>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_complete_identify_result() {
        let json = r#"{
            "common_name": "Monstera",
            "scientific_name": "Monstera deliciosa",
            "confidence": 0.95,
            "summary": "A popular tropical houseplant",
            "care_profile": {
                "watering_interval_days": 7,
                "light_needs": "bright indirect",
                "difficulty": "easy",
                "pet_safety": "toxic to cats and dogs",
                "growth_speed": "moderate",
                "soil_type": "well-draining potting mix",
                "soil_moisture": "slightly moist"
            }
        }"#;

        let result: IdentifyResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.common_name, "Monstera");
        assert_eq!(result.scientific_name, "Monstera deliciosa");
        assert!((result.confidence.unwrap() - 0.95).abs() < f64::EPSILON);
        assert_eq!(
            result.summary.as_deref(),
            Some("A popular tropical houseplant")
        );
        let care = result.care_profile.unwrap();
        assert_eq!(care.watering_interval_days, Some(7));
        assert_eq!(care.light_needs.as_deref(), Some("bright indirect"));
    }

    #[test]
    fn deserialize_missing_optional_fields() {
        let json = r#"{
            "common_name": "Snake Plant",
            "scientific_name": "Dracaena trifasciata"
        }"#;

        let result: IdentifyResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.common_name, "Snake Plant");
        assert_eq!(result.scientific_name, "Dracaena trifasciata");
        assert!(result.confidence.is_none());
        assert!(result.summary.is_none());
        assert!(result.care_profile.is_none());
    }

    #[test]
    fn deserialize_unparseable_response() {
        let json = r#"{"not_a_plant": true}"#;
        let result = serde_json::from_str::<IdentifyResult>(json);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_identify_result_round_trip() {
        let result = IdentifyResult {
            common_name: "Monstera".to_string(),
            scientific_name: "Monstera deliciosa".to_string(),
            confidence: Some(0.95),
            summary: Some("A tropical houseplant".to_string()),
            care_profile: Some(CareProfile {
                watering_interval_days: Some(7),
                light_needs: Some("bright indirect".to_string()),
                difficulty: Some("easy".to_string()),
                pet_safety: None,
                growth_speed: None,
                soil_type: None,
                soil_moisture: None,
            }),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["common_name"], "Monstera");
        assert_eq!(json["scientific_name"], "Monstera deliciosa");
        assert_eq!(json["confidence"], 0.95);
        assert_eq!(json["summary"], "A tropical houseplant");
        assert_eq!(json["care_profile"]["watering_interval_days"], 7);
        assert_eq!(json["care_profile"]["light_needs"], "bright indirect");
        assert!(json["care_profile"]["pet_safety"].is_null());
    }
}
