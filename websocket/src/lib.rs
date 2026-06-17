// Copyright 2023 Salesforce, Inc. All rights reserved.

// WebSocket Frame Counter Policy
//
// Prefixes each upstream text frame with a sequential counter:
//   "hello"  →  "1:hello"
//   "world"  →  "2:world"
//
// Key PDK concepts demonstrated:
// - `on_create`: per-connection state initialization
// - `on_upgrade_upstream`: intercept and transform client→server frames
// - `on_upgrade_downstream`: pass server→client frames through unchanged
// - `on_done`: cleanup / logging when the WebSocket session ends
// - `Decoder::parse` + `Encoder::encode_client`: frame-level manipulation

mod generated;

use anyhow::Result;
use pdk::hl::*;
use pdk::logger::info;
use pdk::websockets::{Decoder, Encoder, Frame, FrameType};

type BoxError = Box<dyn std::error::Error>;

// Upstream handler (client → server)
//
// Waits for complete WebSocket frames, prefixes text frames with
// a counter and forwards all other frame types unchanged.
async fn handle_upstream(mut state: UpstreamState, mut frame_count: u64) -> Result<(), BoxError> {
    let mut remainder = Vec::new();

    loop {
        let mut bytes = remainder.clone();
        bytes.extend_from_slice(&state.bytes());

        let (frames, leftover) = Decoder::parse(bytes);

        if frames.is_empty() {
            // Incomplete frame — wait for more data before processing
            state = state.accumulate().await;
        } else {
            remainder = leftover;

            let transformed: Vec<Frame> = frames
                .into_iter()
                .map(|frame| match frame.frame_type() {
                    FrameType::Text => {
                        frame_count += 1;
                        let original = String::from_utf8_lossy(frame.data());
                        Frame::text(format!("{frame_count}:{original}"), frame.fin())
                    }
                    // Binary and control frames pass through unchanged
                    _ => frame,
                })
                .collect();

            let encoded = Encoder::default().encode_client(transformed);
            state.set_body(&encoded);
            state = state.next().await;
        }
    }
}

// Downstream handler (server → client)
//
// Ensures text frames end with a newline so terminal output stays on its own line.
async fn handle_downstream(mut state: DownstreamState) -> Result<(), BoxError> {
    let mut remainder = Vec::new();

    loop {
        let mut bytes = remainder.clone();
        bytes.extend_from_slice(&state.bytes());

        let (frames, leftover) = Decoder::parse(bytes);

        if frames.is_empty() {
            state = state.accumulate().await;
        } else {
            remainder = leftover;

            let transformed: Vec<Frame> = frames
                .into_iter()
                .map(|frame| match frame.frame_type() {
                    FrameType::Text if !frame.data().ends_with(b"\n") => {
                        let mut text = frame.data().to_vec();
                        text.push(b'\n');
                        Frame::text(text, frame.fin())
                    }
                    _ => frame,
                })
                .collect();

            let encoded = Encoder::default().encode_server(transformed);
            state.set_body(&encoded);
            state = state.next().await;
        }
    }
}

// Per-connection state
#[derive(Clone)]
struct CounterState {
    frame_count: u64,
}

#[pdk::entrypoint]
pub async fn configure(launcher: Launcher) -> Result<()> {
    let handler = FilterBuilder::new()
        .on_create(|| CounterState { frame_count: 0 })
        .on_request(|_req: RequestState| async move { Flow::Continue(()) })
        .on_upgrade_upstream(|state: UpstreamState, State(st): State<CounterState>| {
            handle_upstream(state, st.frame_count)
        })
        .on_upgrade_downstream(|state: DownstreamState, _: State<CounterState>| {
            handle_downstream(state)
        })
        .on_done(|st: CounterState| {
            info!(
                "WebSocket session closed — {count} text frames processed",
                count = st.frame_count
            );
        })
        .build();

    launcher.launch(handler).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdk_unit::{
        UnitFrame, UnitFrameType, UnitHttpRequest, UnitHttpResponse, UnitTest, UnitTestBuilder,
    };

    fn tester() -> UnitTest {
        UnitTestBuilder::default()
            .with_config("{}")
            .with_backend(UnitHttpResponse::upgrade())
            .with_entrypoint(configure)
    }

    #[test]
    fn text_frames_are_prefixed_with_counter() {
        let mut t = tester();
        let conn = t.upgrade(UnitHttpRequest::upgrade()).unwrap();

        conn.client().send_to_server(UnitFrame::text("hello", true));
        let frame = conn.server().next().unwrap();
        assert_eq!(String::from_utf8_lossy(frame.data()), "1:hello");

        conn.client().send_to_server(UnitFrame::text("world", true));
        let frame = conn.server().next().unwrap();
        assert_eq!(String::from_utf8_lossy(frame.data()), "2:world");
    }

    #[test]
    fn binary_frames_pass_through_unchanged() {
        let mut t = tester();
        let conn = t.upgrade(UnitHttpRequest::upgrade()).unwrap();

        let payload = vec![0x01, 0x02, 0x03];
        conn.client()
            .send_to_server(UnitFrame::binary(payload.clone(), true));

        let frame = conn.server().next().unwrap();
        assert_eq!(frame.frame_type(), UnitFrameType::Binary);
        assert_eq!(frame.data(), payload.as_slice());
    }

    #[test]
    fn control_frames_pass_through_unchanged() {
        let mut t = tester();
        let conn = t.upgrade(UnitHttpRequest::upgrade()).unwrap();

        conn.client().send_to_server(UnitFrame::ping());
        assert_eq!(
            conn.server().next().unwrap().frame_type(),
            UnitFrameType::Ping
        );

        conn.client().send_to_server(UnitFrame::pong());
        assert_eq!(
            conn.server().next().unwrap().frame_type(),
            UnitFrameType::Pong
        );
    }

    #[test]
    fn counter_is_not_incremented_by_non_text_frames() {
        let mut t = tester();
        let conn = t.upgrade(UnitHttpRequest::upgrade()).unwrap();

        // Binary frame should NOT increment counter.
        conn.client()
            .send_to_server(UnitFrame::binary(vec![0xFF], true));
        conn.server().next().unwrap();

        // Next text frame should still be "1:", not "2:"
        conn.client().send_to_server(UnitFrame::text("msg", true));
        let frame = conn.server().next().unwrap();
        assert_eq!(String::from_utf8_lossy(frame.data()), "1:msg");
    }

    #[test]
    fn incomplete_frames_are_accumulated_before_processing() {
        let mut t = tester();
        t.set_chunk_size(2); // Force TCP fragmentation
        let conn = t.upgrade(UnitHttpRequest::upgrade()).unwrap();

        conn.client()
            .send_to_server(UnitFrame::text("hello world", true));

        let frame = conn.server().next().unwrap();
        assert_eq!(
            String::from_utf8_lossy(frame.data()),
            "1:hello world",
            "Frame must not be split mid-payload"
        );
    }
}
