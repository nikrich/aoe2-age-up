pub mod engine;
pub mod parser;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOrder {
    pub id: String,
    pub name: String,
    pub civilization: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub difficulty: Option<Difficulty>,
    #[serde(default)]
    pub glyph: Option<String>,
    pub steps: Vec<Step>,
}

impl BuildOrder {
    pub fn to_meta(&self, path: &str) -> BuildOrderMeta {
        BuildOrderMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            civilization: self.civilization.clone(),
            tags: self.tags.clone(),
            description: self.description.clone(),
            difficulty: self.difficulty,
            glyph: self.glyph.clone(),
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Dark,
    Feudal,
    Castle,
    Imperial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub action: String,
    pub at: Trigger,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub villagers_assigned: Option<VillagerAssignment>,
    /// Display-time target for this step (seconds). Used to render "elapsed / target" delta.
    #[serde(default)]
    pub target_time_seconds: Option<u32>,
    /// Age phase this step belongs to. Used for timeline phase coloring.
    #[serde(default)]
    pub phase: Option<Phase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(default)]
    pub time_seconds: Option<u32>,
    #[serde(default)]
    pub villagers: Option<u32>,
    #[serde(default)]
    pub population_min: Option<u32>,
    #[serde(default)]
    pub food_min: Option<u32>,
    #[serde(default)]
    pub wood_min: Option<u32>,
    #[serde(default)]
    pub gold_min: Option<u32>,
    #[serde(default)]
    pub stone_min: Option<u32>,
    #[serde(default)]
    pub mode: TriggerMode,
}

impl Trigger {
    pub fn has_any_condition(&self) -> bool {
        self.time_seconds.is_some()
            || self.villagers.is_some()
            || self.population_min.is_some()
            || self.food_min.is_some()
            || self.wood_min.is_some()
            || self.gold_min.is_some()
            || self.stone_min.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    All,
    Any,
}

impl Default for TriggerMode {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VillagerAssignment {
    pub food: u32,
    pub wood: u32,
    pub gold: u32,
    pub stone: u32,
    #[serde(default)]
    pub idle: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOrderMeta {
    pub id: String,
    pub name: String,
    pub civilization: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub difficulty: Option<Difficulty>,
    pub glyph: Option<String>,
    pub path: String,
}
