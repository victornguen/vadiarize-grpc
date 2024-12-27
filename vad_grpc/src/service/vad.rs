use crate::settings::settings::VadSettings;

pub struct VadService {
    recognizer_8k: silero_vad::Recognizer,
    recognizer_16k: silero_vad::Recognizer,
}

impl VadService {
    pub fn new(settings: &VadSettings) -> vad_grpc_server::Result<Self> {
        fn vad_params(settings: &VadSettings, sample_rate: usize) -> silero_vad::VadParams {
            silero_vad::VadParams {
                sample_rate,
                frame_size: settings.frame_size,
                threshold: settings.threshold,
                min_silence_duration_ms: settings.min_silence_duration_ms,
                speech_pad_ms: settings.speech_pad_ms,
                min_speech_duration_ms: settings.min_speech_duration_ms,
                max_speech_duration_s: settings.max_speech_duration_s,
                ..Default::default()
            }
        }

        let vad_params_8k = vad_params(settings, 8000);
        let vad_params_16k = vad_params(settings, 16000);

        let recognizer_8k =
            silero_vad::Recognizer::new(settings.model_path.as_str(), vad_params_8k, settings.sessions_num)?;
        let recognizer_16k =
            silero_vad::Recognizer::new(settings.model_path.as_str(), vad_params_16k, settings.sessions_num)?;
        Ok(Self {
            recognizer_8k,
            recognizer_16k,
        })
    }

    pub fn recognize(&self, audio: Vec<i16>, sample_rate: u32) -> vad_grpc_server::Result<Vec<silero_vad::TimeStamp>> {
        match sample_rate {
            8000 => Ok(self.recognizer_8k.process(&*audio)?),
            16000 => Ok(self.recognizer_16k.process(&*audio)?),
            _ => Err(vad_grpc_server::VadServiceError::InvalidAudio("Unsupported sample rate".to_string())),
        }
    }
}
