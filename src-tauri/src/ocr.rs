use glypho::{Device, OnnxConfig, OnnxEngine, QualityMode};
use std::sync::Arc;
use tauri::Manager;

pub struct OcrState {
    engine: Arc<OnnxEngine>,
}

pub fn init_ocr(app: &tauri::AppHandle) -> Result<OcrState, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    let models_dir = app_data_dir.join("glypho-models");
    std::fs::create_dir_all(&models_dir)?;

    // Создаём конфиг сразу с нужной директорией — без set_var
    let mut config = OnnxConfig::new(models_dir);
    config.quality = QualityMode::Maximum;
    config.device = Device::Auto;

    let engine = OnnxEngine::new(config)?;
    engine.warmup(&["en".to_string()])?;

    Ok(OcrState {
        engine: Arc::new(engine),
    })
}
