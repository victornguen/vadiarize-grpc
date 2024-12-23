use crate::pb::vad_grpc::vad_recognizer_server::VadRecognizer;
use crate::pb::vad_grpc::{SpeechInterval, VadRequest, VadResponse};
use crate::settings::settings::Settings;
use crate::VadService;
use futures::Stream;
use std::pin::Pin;
use tonic::{Request, Response, Status, Streaming};

pub struct VadServiceController {
    vad: VadService,
}

impl VadServiceController {
    pub fn new(settings: &Settings) -> vad_grpc_server::Result<Self> {
        let vad = VadService::new(&settings.vad)?;
        Ok(Self { vad })
    }
}

#[tonic::async_trait]
impl VadRecognizer for VadServiceController {
    async fn detect(&self, request: Request<VadRequest>) -> Result<Response<VadResponse>, Status> {
        let request = request.into_inner();
        // transform request.audio, which is a Vec<u8>, into a Vec<i16> by union 2 bytes into 1 float
        let audio = request
            .audio
            .chunks(2)
            .map(|chunk| {
                let mut bytes = [0; 2];
                bytes.copy_from_slice(chunk);
                i16::from_ne_bytes(bytes)
            })
            .collect();
        let result = self.vad.recognize(audio).map_err(|e| Status::internal(e.to_string()))?;
        let intervals = result
            .into_iter()
            .map(|ts| {
                let start = ts.start;
                let end = ts.end;
                SpeechInterval {
                    start_s: start,
                    end_s: end,
                }
            })
            .collect();
        let response = VadResponse {
            intervals,
            request_id: None,
        };
        Ok(Response::new(response))
    }

    type DetectStreamStream = Pin<Box<dyn Stream<Item = Result<VadResponse, Status>> + Send>>;

    async fn detect_stream(
        &self,
        request: Request<Streaming<VadRequest>>,
    ) -> Result<Response<Self::DetectStreamStream>, Status> {
        todo!()
    }
}
