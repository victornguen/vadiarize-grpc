use crate::utils::{TimeStamp, VadParams};
use crate::{error, silero, vad_iter, OnnxSession};
use lockfree_object_pool::MutexObjectPool;
use ort::session::builder::GraphOptimizationLevel;
use std::iter::Cycle;
use std::sync::Arc;
use std::vec::IntoIter;

pub struct Recognizer {
    vad_iter_pool: Arc<MutexObjectPool<vad_iter::VadIter>>,
    sessions: Arc<parking_lot::Mutex<Cycle<IntoIter<Arc<OnnxSession>>>>>,
}

impl Recognizer {
    pub fn new(model_path: &str, vad_params: VadParams, sessions_num: u8) -> Result<Self, error::VadError> {
        let onnx_sessions = (0..sessions_num)
            .map(|_| Recognizer::make_onnx_session(model_path).map(Arc::new))
            .collect::<Result<Vec<_>, _>>()?;

        let sessions_iter = Arc::new(parking_lot::Mutex::new(onnx_sessions.into_iter().cycle()));

        let sample_rate = vad_params.sample_rate.into();

        let sessions_iter_clone = sessions_iter.clone();
        let vad_iter_pool = MutexObjectPool::<vad_iter::VadIter>::new(
            move || {
                let session = sessions_iter_clone.lock().next().expect("no onnx sessions to cycle");
                let silero = silero::SileroSession::new(session, sample_rate).expect("error creating Silero session");
                vad_iter::VadIter::new(silero, vad_params.clone())
            },
            |_| {},
        );

        Ok(Self {
            vad_iter_pool: Arc::new(vad_iter_pool),
            sessions: sessions_iter,
        })
    }

    fn make_onnx_session(model_path: &str) -> crate::Result<OnnxSession> {
        let session = OnnxSession::builder()?
            .with_inter_threads(1)?
            // .with_execution_providers(vec![CUDAExecutionProvider::default().build()]).expect("onnx session providers error")
            .with_intra_threads(1)?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .commit_from_file(model_path)?;
        Ok(session)
    }

    pub fn process(&self, samples: &[i16]) -> Result<Vec<TimeStamp>, error::VadError> {
        let mut vad = self.vad_iter_pool.pull();
        vad.process(samples)?;
        Ok(vad.speeches())
    }
}
