use ort::execution_providers::CUDAExecutionProvider;
use ort::session::builder::GraphOptimizationLevel;
use silero_vad::tools::time::timed;
use silero_vad::{OnnxSession, Recognizer};

fn main() {
    let model_path =
        std::env::var("SILERO_MODEL_PATH").unwrap_or_else(|_| String::from("silero_vad/model/silero_vad_3.onnx"));
    let audio_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("out_stereo_ch1_16k.wav"));
    let mut wav_reader = hound::WavReader::open(audio_path).unwrap();
    let sr = wav_reader.spec().sample_rate;
    let sample_rate = match &sr {
        8000 => utils::SampleRate::EightKHz,
        16000 => utils::SampleRate::SixteenKHz,
        _ => panic!("Unsupported sample rate. Expect 8 kHz or 16 kHz."),
    };
    if wav_reader.spec().sample_format != hound::SampleFormat::Int {
        panic!("Unsupported sample format. Expect Int.");
    }
    let content = wav_reader
        .samples()
        .filter_map(|x| x.ok())
        .collect::<Vec<i16>>()
        .repeat(10);
    assert!(!content.is_empty());

    let vad_params = utils::VadParams {
        sample_rate: sample_rate.into(),
        ..Default::default()
    };

    let recognizer = Recognizer::new(&model_path, vad_params, 3).unwrap();

    // let onnx_session = OnnxSession::builder().expect("onnx session builder error")
    //     .with_inter_threads(1).expect("onnx session threads error")
    //     // .with_execution_providers(vec![CUDAExecutionProvider::default().build()]).expect("onnx session providers error")
    //     .with_intra_threads(1).expect("onnx session threads error")
    //     .with_optimization_level(GraphOptimizationLevel::Level3).expect("onnx session optimization error")
    //     .commit_from_file(model_path).expect("onnx session commit error");
    // let onnx_session = std::sync::Arc::new(onnx_session);
    // let silero = || silero::SileroSession::new(onnx_session, sample_rate).unwrap();
    // let silero = timed("Silero create", silero);

    // let mut vad_iterator = vad_iter::VadIter::new(silero, vad_params);
    let f = || recognizer.process(&content).unwrap();
    let res = timed("VAD", f);

    res.iter().for_each(|ts| println!("{} - {}", ts.start, ts.end));

    // for timestamp in vad_iterator.speeches() {
    //     println!("\n{}", timestamp);
    // }
    println!("Finished.");
}
