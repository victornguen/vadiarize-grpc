use crate::pb::vad_grpc_v1::SpeechInterval;
use silero_vad::TimeStamp;

pub fn timestamps_to_speech_intervals(timestamps: &[TimeStamp]) -> Vec<SpeechInterval> {
    timestamps
        .iter()
        .map(|ts| SpeechInterval {
            start_s: ts.start,
            end_s: ts.end,
        })
        .collect()
}

pub fn timestamp_to_speech_interval(timestamp: &TimeStamp) -> SpeechInterval {
    SpeechInterval {
        start_s: timestamp.start,
        end_s: timestamp.end,
    }
}
