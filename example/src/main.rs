use silero_vad::tools::time::timed;
use silero_vad::utils;
use silero_vad::utils::SampleRate;
use silero_vad::Recognizer;

fn main() {
    let model_path =
        std::env::var("SILERO_MODEL_PATH").unwrap_or_else(|_| String::from("silero_vad/model/silero_vad_3.onnx"));
    let audio_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("out_stereo_ch1_16k.wav"));
    let mut wav_reader = hound::WavReader::open(audio_path).unwrap();

    let sample_rate = SampleRate::from(wav_reader.spec().sample_rate as usize);

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

    let f = || recognizer.process(&content).unwrap();
    let res = timed("VAD", f);

    res.iter().for_each(|ts| println!("{} - {}", ts.start, ts.end));

    println!("Finished.");
}
