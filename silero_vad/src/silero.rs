use crate::{utils, OnnxSession};
use ndarray::{s, Array, Array2, ArrayBase, ArrayD, Dim, IxDynImpl, OwnedRepr};
use std::sync::Arc;

#[derive(Debug)]
pub struct SileroSession {
    session: Arc<OnnxSession>,
    sample_rate: ArrayBase<OwnedRepr<i64>, Dim<[usize; 1]>>,
    state: ArrayBase<OwnedRepr<f32>, Dim<IxDynImpl>>,
}

impl SileroSession {
    pub fn new(
        session: Arc<OnnxSession>,
        sample_rate: utils::SampleRate,
        // model_path: impl AsRef<Path>,
    ) -> Result<Self, ort::Error> {
        // let session = OnnxSession::builder()?.commit_from_file(model_path)?;
        let state = ArrayD::<f32>::zeros([2, 1, 128].as_slice());
        let sample_rate = Array::from_shape_vec([1], vec![sample_rate.into()]).unwrap();
        Ok(Self {
            session,
            sample_rate,
            state,
        })
    }

    pub fn reset(&mut self) {
        self.state = ArrayD::<f32>::zeros([2, 1, 128].as_slice());
    }

    pub fn calc_level(&mut self, audio_frame: &[i16]) -> Result<f32, ort::Error> {
        let data = audio_frame
            .iter()
            .map(|x| (*x as f32) / (i16::MAX as f32))
            .collect::<Vec<_>>();
        let mut frame = Array2::<f32>::from_shape_vec([1, data.len()], data).unwrap();
        frame = frame.slice(s![.., ..480]).to_owned();
        let inps = ort::inputs![frame, std::mem::take(&mut self.state), self.sample_rate.clone(),]?;
        let res = self.session.run(ort::session::SessionInputs::ValueSlice::<3>(&inps))?;
        self.state = res["stateN"].try_extract_tensor().unwrap().to_owned();
        Ok(*res["output"].try_extract_raw_tensor::<f32>()?.1.first().unwrap())
    }
}
