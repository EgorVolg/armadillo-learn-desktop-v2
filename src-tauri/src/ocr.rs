use glypho::{Device, OnnxConfig, OnnxEngine, QualityMode, RecognitionOptions};
use std::error::Error;
use std::sync::Arc;
use tauri::Manager;

pub struct OcrState {
    engine: Arc<OnnxEngine>,
}

impl OcrState {
    /// Создаёт новый экземпляр OcrState с предзагруженным движком.
    pub fn init(app: &tauri::AppHandle) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let app_data_dir = app.path().app_data_dir()?;
        let models_dir = app_data_dir.join("glypho-models");
        std::fs::create_dir_all(&models_dir)?;

        let mut config = OnnxConfig::new(models_dir);
        config.quality = QualityMode::Maximum;
        config.device = Device::Auto;

        let engine = OnnxEngine::new(config)?;
        engine.warmup(&["en".to_string()])?;

        Ok(OcrState {
            engine: Arc::new(engine),
        })
    }

    /// Синхронное распознавание из байтов.
    /// Блокирует текущий поток — не вызывайте из UI-потока напрямую.
    pub fn recognize_bytes(
        &self,
        image_bytes: &[u8],
        file_name: &str,
        languages: Vec<String>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let options = RecognitionOptions {
            languages,
            ..Default::default()
        };

        let doc = self
            .engine
            .recognize_bytes(image_bytes, file_name, &options)?;
        Ok(doc.text)
    }
}
