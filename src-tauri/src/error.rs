use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Build order not found: {0}")]
    BuildOrderNotFound(String),

    #[error("Invalid build order: {0}")]
    InvalidBuildOrder(String),

    #[error("No build order loaded")]
    NoBuildOrderLoaded,

    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Capture error: {0}")]
    Capture(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
