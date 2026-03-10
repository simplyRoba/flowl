use serde::Serialize;
use sqlx::SqlitePool;

use crate::api::error::ApiError;

// --- Context structs ---

#[derive(Serialize)]
pub struct PlantContext {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    current_state: CurrentState,
    care_preferences: CarePreferences,
    recent_care_events: Vec<CareEventContext>,
}

#[derive(Serialize)]
pub struct CurrentState {
    watering_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_watered: Option<String>,
}

#[derive(Serialize)]
pub struct CarePreferences {
    light_needs: String,
    watering_interval_days: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pet_safety: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    growth_speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    soil_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    soil_moisture: Option<String>,
}

#[derive(Serialize)]
pub struct CareEventContext {
    event_type: String,
    date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

// --- Query structs ---

#[derive(sqlx::FromRow)]
struct PlantContextRow {
    name: String,
    species: Option<String>,
    location_name: Option<String>,
    light_needs: String,
    watering_interval_days: i64,
    last_watered: Option<String>,
    difficulty: Option<String>,
    pet_safety: Option<String>,
    growth_speed: Option<String>,
    soil_type: Option<String>,
    soil_moisture: Option<String>,
    notes: Option<String>,
}

#[derive(sqlx::FromRow)]
struct CareEventRow {
    event_type: String,
    occurred_at: String,
    notes: Option<String>,
}

// --- Builders ---

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist, or
/// `ApiError::InternalError` on database failures.
pub async fn build_plant_context(
    pool: &SqlitePool,
    plant_id: i64,
) -> Result<PlantContext, ApiError> {
    let row = sqlx::query_as::<_, PlantContextRow>(
        "SELECT p.name, p.species, l.name AS location_name, p.light_needs, \
         p.watering_interval_days, \
         (SELECT MAX(occurred_at) FROM care_events WHERE plant_id = p.id AND event_type = 'watered') AS last_watered, \
         p.difficulty, p.pet_safety, p.growth_speed, p.soil_type, p.soil_moisture, p.notes \
         FROM plants p LEFT JOIN locations l ON p.location_id = l.id \
         WHERE p.id = ?",
    )
    .bind(plant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?
    .ok_or(ApiError::NotFound("PLANT_NOT_FOUND"))?;

    let (watering_status, _) = crate::api::plants::compute_watering_status(
        row.last_watered.as_deref(),
        row.watering_interval_days,
    );

    let events = sqlx::query_as::<_, CareEventRow>(
        "SELECT event_type, occurred_at, notes FROM care_events \
         WHERE plant_id = ? ORDER BY occurred_at DESC LIMIT 20",
    )
    .bind(plant_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    let recent_care_events = events
        .into_iter()
        .map(|e| CareEventContext {
            event_type: e.event_type,
            date: e
                .occurred_at
                .get(..10)
                .unwrap_or(&e.occurred_at)
                .to_string(),
            notes: e.notes,
        })
        .collect();

    Ok(PlantContext {
        name: row.name,
        species: row.species,
        location_name: row.location_name,
        notes: row.notes,
        current_state: CurrentState {
            watering_status,
            last_watered: row
                .last_watered
                .as_deref()
                .map(|d| d.get(..10).unwrap_or(d).to_string()),
        },
        care_preferences: CarePreferences {
            light_needs: row.light_needs,
            watering_interval_days: row.watering_interval_days,
            difficulty: row.difficulty,
            pet_safety: row.pet_safety,
            growth_speed: row.growth_speed,
            soil_type: row.soil_type,
            soil_moisture: row.soil_moisture,
        },
        recent_care_events,
    })
}

pub fn build_chat_system_prompt(context: &PlantContext, locale: &str) -> String {
    let context_json = serde_json::to_string_pretty(context).unwrap_or_else(|_| "{}".to_string());

    let lang_instruction = locale_instruction(locale);

    format!(
        "You are flowl, a plant care assistant embedded in a self-hosted plant management app. \
         You help users with plant health diagnosis, watering advice, and general care questions.\n\n\
         You have access to the user's plant data and recent care history (provided below as JSON). \
         Use this context to give specific, personalized advice rather than generic answers.\n\n\
         The care_preferences section describes the desired conditions for this plant, not its current state.\n\n\
         Be friendly and casual — use informal language (e.g. \"du\" in German, \"tú\" in Spanish). \
         Be concise and practical — 2-4 short paragraphs max. \
         Use plain text only — no markdown, no bold, no headers, no code blocks. \
         Use simple dashes (- ) for lists. \
         If you're unsure about a diagnosis, say so and suggest what to look for.\n\n\
         Do not answer questions unrelated to plant care.\n\n\
         Plant context:\n{context_json}\n\n\
         {lang_instruction}"
    )
}

pub fn build_summarize_system_prompt(
    plant_name: &str,
    species: Option<&str>,
    locale: &str,
) -> String {
    let species_part = species.map(|s| format!(" ({s})")).unwrap_or_default();

    let lang_instruction = locale_instruction(locale);

    format!(
        "Summarize the following conversation about the plant \"{plant_name}\"{species_part} \
         into a 1-3 sentence care journal note. Focus on diagnoses, advice given, \
         and actions recommended.\n\n\
         {lang_instruction}"
    )
}

pub fn build_identify_prompt(locale: &str) -> String {
    let lang_instruction = match locale {
        "en" => "Respond in English.".to_string(),
        _ => format!(
            "Respond in the language with locale code \"{locale}\". \
             Use that language for the common_name and summary fields. \
             Keep scientific_name in Latin."
        ),
    };

    format!(
        "Identify this plant from the photo(s). Provide your top 3 most likely identifications, \
         ranked by confidence (highest first). For each, include the common name, scientific name, \
         your confidence level, a short summary of the species, and a care profile with typical \
         care requirements. {lang_instruction}"
    )
}

pub fn locale_instruction(locale: &str) -> String {
    match locale {
        "en" => "Respond in English.".to_string(),
        "de" => "Respond in German.".to_string(),
        "es" => "Respond in Spanish.".to_string(),
        _ => format!("Respond in the language with locale code \"{locale}\"."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_plant_context_serializes_with_nested_groups() {
        let context = PlantContext {
            name: "Monstera".to_string(),
            species: Some("Monstera deliciosa".to_string()),
            location_name: Some("Living Room".to_string()),
            notes: None,
            current_state: CurrentState {
                watering_status: "ok".to_string(),
                last_watered: Some("2026-02-20".to_string()),
            },
            care_preferences: CarePreferences {
                light_needs: "indirect".to_string(),
                watering_interval_days: 10,
                difficulty: Some("easy".to_string()),
                pet_safety: Some("toxic".to_string()),
                growth_speed: Some("moderate".to_string()),
                soil_type: Some("standard".to_string()),
                soil_moisture: Some("moderate".to_string()),
            },
            recent_care_events: vec![],
        };

        let json = serde_json::to_string_pretty(&context).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify nested groups exist
        assert!(value.get("current_state").is_some());
        assert!(value.get("care_preferences").is_some());

        // Verify fields are inside the groups, not at top level
        let current_state = value.get("current_state").unwrap();
        assert_eq!(current_state["watering_status"], "ok");
        assert_eq!(current_state["last_watered"], "2026-02-20");

        let prefs = value.get("care_preferences").unwrap();
        assert_eq!(prefs["light_needs"], "indirect");
        assert_eq!(prefs["watering_interval_days"], 10);
        assert_eq!(prefs["difficulty"], "easy");

        // Verify care preference fields are NOT at top level
        assert!(value.get("light_needs").is_none());
        assert!(value.get("watering_interval_days").is_none());
        assert!(value.get("watering_status").is_none());
    }

    #[test]
    fn chat_system_prompt_includes_care_preferences_clarification() {
        let context = PlantContext {
            name: "Fern".to_string(),
            species: None,
            location_name: None,
            notes: None,
            current_state: CurrentState {
                watering_status: "due".to_string(),
                last_watered: None,
            },
            care_preferences: CarePreferences {
                light_needs: "low".to_string(),
                watering_interval_days: 3,
                difficulty: None,
                pet_safety: None,
                growth_speed: None,
                soil_type: None,
                soil_moisture: None,
            },
            recent_care_events: vec![],
        };
        let prompt = build_chat_system_prompt(&context, "en");
        assert!(prompt.contains(
            "The care_preferences section describes the desired conditions for this plant, not its current state."
        ));
    }

    #[test]
    fn chat_system_prompt_contains_plant_context() {
        let context = PlantContext {
            name: "Monstera".to_string(),
            species: Some("Monstera deliciosa".to_string()),
            location_name: Some("Living Room".to_string()),
            notes: Some("Bought from nursery".to_string()),
            current_state: CurrentState {
                watering_status: "ok".to_string(),
                last_watered: Some("2026-02-20".to_string()),
            },
            care_preferences: CarePreferences {
                light_needs: "indirect".to_string(),
                watering_interval_days: 10,
                difficulty: Some("easy".to_string()),
                pet_safety: Some("toxic".to_string()),
                growth_speed: Some("moderate".to_string()),
                soil_type: Some("standard".to_string()),
                soil_moisture: Some("moderate".to_string()),
            },
            recent_care_events: vec![CareEventContext {
                event_type: "watered".to_string(),
                date: "2026-02-20".to_string(),
                notes: None,
            }],
        };
        let prompt = build_chat_system_prompt(&context, "en");
        assert!(prompt.contains("flowl"));
        assert!(prompt.contains("Monstera"));
        assert!(prompt.contains("Monstera deliciosa"));
        assert!(prompt.contains("Living Room"));
        assert!(prompt.contains("watered"));
        assert!(prompt.contains("Respond in English"));
    }

    #[test]
    fn chat_system_prompt_locale_german() {
        let context = PlantContext {
            name: "Fern".to_string(),
            species: None,
            location_name: None,
            notes: None,
            current_state: CurrentState {
                watering_status: "due".to_string(),
                last_watered: None,
            },
            care_preferences: CarePreferences {
                light_needs: "low".to_string(),
                watering_interval_days: 3,
                difficulty: None,
                pet_safety: None,
                growth_speed: None,
                soil_type: None,
                soil_moisture: None,
            },
            recent_care_events: vec![],
        };
        let prompt = build_chat_system_prompt(&context, "de");
        assert!(prompt.contains("Respond in German"));
    }

    #[test]
    fn chat_system_prompt_no_optional_fields() {
        let context = PlantContext {
            name: "Unknown".to_string(),
            species: None,
            location_name: None,
            notes: None,
            current_state: CurrentState {
                watering_status: "ok".to_string(),
                last_watered: None,
            },
            care_preferences: CarePreferences {
                light_needs: "indirect".to_string(),
                watering_interval_days: 7,
                difficulty: None,
                pet_safety: None,
                growth_speed: None,
                soil_type: None,
                soil_moisture: None,
            },
            recent_care_events: vec![],
        };
        let prompt = build_chat_system_prompt(&context, "en");
        assert!(prompt.contains("Unknown"));
        assert!(prompt.contains("recent_care_events"));
    }

    #[test]
    fn summarize_system_prompt_with_species() {
        let prompt = build_summarize_system_prompt("Monstera", Some("Monstera deliciosa"), "en");
        assert!(prompt.contains("Monstera"));
        assert!(prompt.contains("Monstera deliciosa"));
        assert!(prompt.contains("1-3 sentence"));
        assert!(prompt.contains("Respond in English"));
    }

    #[test]
    fn summarize_system_prompt_without_species() {
        let prompt = build_summarize_system_prompt("My Plant", None, "es");
        assert!(prompt.contains("My Plant"));
        assert!(!prompt.contains("()"));
        assert!(prompt.contains("Respond in Spanish"));
    }

    #[test]
    fn locale_instruction_known_locales() {
        assert_eq!(locale_instruction("en"), "Respond in English.");
        assert_eq!(locale_instruction("de"), "Respond in German.");
        assert_eq!(locale_instruction("es"), "Respond in Spanish.");
    }

    #[test]
    fn locale_instruction_unknown_locale() {
        let result = locale_instruction("fr");
        assert!(result.contains("fr"));
    }
}
