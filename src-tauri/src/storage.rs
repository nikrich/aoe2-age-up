use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;

use crate::build_order::BuildOrderMeta;
use crate::build_order::parser::load_build_order;
use crate::state::{Calibration, Settings};

pub struct Storage {
    app_data_dir: PathBuf,
}

impl Storage {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        let storage = Self { app_data_dir };
        storage.ensure_directories()?;
        Ok(storage)
    }

    fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            self.build_orders_dir(),
            self.build_orders_dir().join("user"),
            self.calibration_dir(),
        ];
        for dir in &dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            }
        }
        Ok(())
    }

    pub fn build_orders_dir(&self) -> PathBuf {
        self.app_data_dir.join("build-orders")
    }

    fn calibration_dir(&self) -> PathBuf {
        self.app_data_dir.join("calibration")
    }

    fn settings_path(&self) -> PathBuf {
        self.app_data_dir.join("settings.json")
    }

    pub fn copy_bundled_build_orders(&self, bundled_dir: &Path) -> Result<()> {
        if !bundled_dir.exists() {
            return Ok(());
        }
        let target_dir = self.build_orders_dir();
        for entry in std::fs::read_dir(bundled_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml" || ext == "json") {
                let target = target_dir.join(entry.file_name());
                std::fs::copy(&path, &target)?;
                info!("Copied bundled build order: {}", entry.file_name().to_string_lossy());
            }
        }
        Ok(())
    }

    pub fn list_build_orders(&self) -> Result<Vec<BuildOrderMeta>> {
        let mut metas = Vec::new();
        self.scan_build_orders_in(&self.build_orders_dir(), &mut metas)?;
        let user_dir = self.build_orders_dir().join("user");
        if user_dir.exists() {
            self.scan_build_orders_in(&user_dir, &mut metas)?;
        }
        Ok(metas)
    }

    fn scan_build_orders_in(&self, dir: &Path, metas: &mut Vec<BuildOrderMeta>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext, "yaml" | "yml" | "json") {
                    match load_build_order(&path) {
                        Ok(bo) => metas.push(bo.to_meta(&path.to_string_lossy())),
                        Err(e) => tracing::warn!("Skipping {}: {}", path.display(), e),
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_settings(&self) -> Result<Settings> {
        let path = self.settings_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Settings::default())
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(self.settings_path(), content)?;
        Ok(())
    }

    pub fn load_calibration(&self, resolution: (u32, u32)) -> Result<Calibration> {
        let path = self.calibration_dir().join(format!("{}x{}.json", resolution.0, resolution.1));
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Calibration::default())
        }
    }

    pub fn save_calibration(&self, calibration: &Calibration) -> Result<()> {
        let path = self.calibration_dir().join(format!(
            "{}x{}.json",
            calibration.resolution.0, calibration.resolution.1
        ));
        let content = serde_json::to_string_pretty(calibration)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
