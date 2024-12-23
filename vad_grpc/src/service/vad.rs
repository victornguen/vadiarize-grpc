use crate::settings::settings::VadSettings;

pub struct VadService {
    recognizer: silero_vad::Recognizer,
}

impl VadService {
    pub fn new(settings: &VadSettings) -> vad_grpc_server::Result<Self> {
        let vad_params = silero_vad::VadParams {
            sample_rate: 16000,
            ..Default::default()
        };

        let recognizer = silero_vad::Recognizer::new(settings.model_path.as_str(), vad_params, settings.sessions_num)?;
        Ok(Self { recognizer })
    }

    pub fn recognize(&self, audio: Vec<i16>) -> vad_grpc_server::Result<Vec<silero_vad::TimeStamp>> {
        let result = self.recognizer.process(&*audio)?;
        Ok(result)
    }
}
