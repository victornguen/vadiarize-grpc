use voice_stream::{cpal, VoiceStreamBuilder, WebRtcVoiceActivityProfile};
use voice_stream::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {

    let (tx, rx) = std::sync::mpsc::channel();

    let host = cpal::default_host();

    let select_device = "default";

    // Set up the input device and stream with the default input config.
    let device = if select_device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()
            .expect("Failed to get input devices")
            .find(|x| x.name().map(|y| y == select_device).unwrap_or(false))
    }
        .expect("failed to find input device");

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");

    let voice_stream = VoiceStreamBuilder::new(config, device, tx)
        .with_sound_buffer_until_size(1024)
        .with_voice_detection_silero_voice_threshold(0.5)
        .with_voice_detection_webrtc_profile(WebRtcVoiceActivityProfile::AGGRESSIVE)
        .build()
        .unwrap();

    for voice in voice_stream {
        println!("Voice detected: {:?}", voice);
    }
}
