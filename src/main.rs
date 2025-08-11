use gstreamer::prelude::*;
use anyhow::Error;

fn main() -> Result<(), Error> {
    // 1. Initialize GStreamer
    gstreamer::init()?;

    // 2. Create the GStreamer elements
    // srtclientsrc uri=srt://192.168.1.55:8888
    let source = gstreamer::ElementFactory::make("srtsrc")
        .name("source")
        .build()?;
    source.set_property("uri", "srt://0.0.0.0:9876?mode=listener");

    // decodebin
    let decoder = gstreamer::ElementFactory::make("decodebin")
        .name("decoder")
        .build()?;

    // autovideosink
    let video_sink = gstreamer::ElementFactory::make("autovideosink")
        .name("video_sink")
        .build()?;

    // 3. Create the pipeline to hold the elements
    let pipeline = gstreamer::Pipeline::with_name("test-pipeline");
    pipeline.add_many([&source, &decoder, &video_sink])?;

    // 4. Link the source to the decoder.
    // The link from decoder to the sink will be handled dynamically.
    source.link(&decoder)?;

    // 5. Handle the dynamic pad creation from `decodebin`
    // This closure will be called when `decodebin` creates a new pad
    decoder.connect_pad_added(move |_, src_pad| {
        println!("Received new pad '{:?}' from '{:?}'", src_pad.name(), src_pad.parent_element().unwrap().name());

        let sink_pad = video_sink.static_pad("sink").expect("Failed to get static sink pad from videosink");
        if sink_pad.is_linked() {
            println!("Sink pad is already linked. Ignoring.");
            return;
        }

        // Check the new pad's type
        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get structure of new pad caps.");

        if new_pad_struct.name().starts_with("video/x-raw") {
            println!("Pad is a raw video pad. Linking...");
            let res = src_pad.link(&sink_pad);
            if res.is_err() {
                println!("Failed to link pads: {:?}", res);
            }
        } else {
            println!("Pad is not a raw video pad. Ignoring.");
        }
    });

    // 6. Start playing
    pipeline.set_state(gstreamer::State::Playing)?;

    // 7. Wait until an error or end-of-stream (EOS) occurs
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                break;
            }
            MessageView::Eos(_) => {
                println!("End-Of-Stream reached.");
                break;
            }
            _ => (),
        }
    }

    // 8. Clean up
    pipeline.set_state(gstreamer::State::Null)?;
    Ok(())
}
