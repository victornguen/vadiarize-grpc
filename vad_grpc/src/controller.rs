use crate::pb::vad_grpc_v1::vad_recognizer_server::VadRecognizer;
use crate::pb::vad_grpc_v1::vad_stream_request::Content;
use crate::pb::vad_grpc_v1::{AudioConfig, AudioType, SpeechInterval, VadRequest, VadResponse, VadStreamRequest};
use crate::settings::settings::Settings;
use crate::tools::grpc::timestamp_to_speech_interval;
use crate::{tools, VadService};
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use vad_grpc_server::VadServiceError;

pub struct VadServiceController {
    vad: Arc<VadService>,
}

impl VadServiceController {
    pub fn new(settings: &Settings) -> vad_grpc_server::Result<Self> {
        let vad = Arc::new(VadService::new(&settings.vad)?);
        Ok(Self { vad })
    }

    fn get_audio_and_config_from_request(request: &VadRequest) -> Result<(Vec<i16>, AudioConfig), Status> {
        let config = request
            .config
            .ok_or_else(|| Status::invalid_argument("No config provided"))?;

        let audio = Self::transform_audio_to_i16(&request.audio, &config)?;

        Ok((audio, config))
    }

    fn transform_audio_to_i16(audio: &[u8], config: &AudioConfig) -> Result<Vec<i16>, Status> {
        match config.audio_type() {
            AudioType::RawPcmS16le => Ok(tools::wav::bytes_to_i16(audio)),
            AudioType::RawPcmS16be => {
                let bytes = tools::transcode::pcm_s16be_to_pcm_s16le(audio);
                Ok(tools::wav::bytes_to_i16(&bytes))
            }
            AudioType::WavPcmS16le => tools::wav::get_samples_from_wav(audio),
            AudioType::Unspecified => {
                Err(VadServiceError::InvalidAudio("Only pcm_s16le and pcm_s16be are supported".to_string()))
            }
        }
        .map_err(|e| Status::invalid_argument(format!("{}", e)))
    }
}

#[tonic::async_trait]
impl VadRecognizer for VadServiceController {
    async fn detect(&self, request: Request<VadRequest>) -> Result<Response<VadResponse>, Status> {
        let request = request.into_inner();
        // transform request.audio, which is a Vec<u8>, into a Vec<i16> by union 2 bytes into 1 float

        let (audio, config) = Self::get_audio_and_config_from_request(&request)?;

        let result = self
            .vad
            .recognize(audio, config.sample_rate as u32)
            .map_err(|e| Status::internal(e.to_string()))?;
        let intervals = result.iter().map(timestamp_to_speech_interval).collect();
        let response = VadResponse {
            intervals,
            request_id: None,
        };
        Ok(Response::new(response))
    }

    type DetectStreamStream = Pin<Box<dyn Stream<Item = Result<VadResponse, Status>> + Send>>;

    async fn detect_stream(
        &self,
        request: Request<Streaming<VadStreamRequest>>,
    ) -> Result<Response<Self::DetectStreamStream>, Status> {
        // get first message from stream
        let mut stream = request.into_inner();
        let first_message = stream
            .message()
            .await?
            .and_then(|m| m.content)
            .ok_or_else(|| Status::invalid_argument("No messages in stream"))?;
        let config = match first_message {
            Content::Config(config) => Ok(config),
            Content::Audio(_) => Err(Status::invalid_argument("First message must be config")),
        }?;

        let vad = self.vad.clone();

        let response = stream.map(move |message| {
            let chunk = match message?.content {
                Some(Content::Audio(audio)) => audio,
                _ => return Err(Status::invalid_argument("Audio message expected")),
            };

            let audio = Self::transform_audio_to_i16(&chunk.audio, &config)?;

            let result = vad
                .recognize(audio, config.sample_rate as u32)
                .map_err(|e| Status::internal(e.to_string()))?;
            let intervals = result.into_iter().map(|ts| timestamp_to_speech_interval(&ts)).collect();
            let response = VadResponse {
                intervals,
                request_id: Some(chunk.request_id),
            };
            Ok(response)
        });

        Ok(Response::new(Box::pin(response) as Self::DetectStreamStream))
    }
}
