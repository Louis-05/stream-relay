use anyhow::Error;
use gstreamer::{Structure, glib::Value, prelude::*};

use crate::{config::Config, srt_stats::SrtStatisticsReport};
mod config;
mod srt_stats;

fn main() -> Result<(), Error> {
    let config = Config::load_from_env()?;
    // 1. Initialize GStreamer
    gstreamer::init()?;

    // 2. Create the GStreamer elements
    // srtclientsrc uri=srt://192.168.1.55:8888
    let srt_source = gstreamer::ElementFactory::make("srtsrc")
        .name("source")
        .property("localaddress", "0.0.0.0")
        .property("keep-listening", true)
        .property_from_str("mode", "listener")
        .property("authentication", true)
        .property("localport", config.input_port as u32)
        .property_from_str("pbkeylen", "32")
        .property("passphrase", &config.passphrase)
        .build()?;

    let tsdemux = gstreamer::ElementFactory::make("tsdemux")
        .name("tsdemux")
        .build()?;

    let multiqueue = gstreamer::ElementFactory::make("multiqueue")
        .name("multiqueue")
        .build()?;

    let h264parse = gstreamer::ElementFactory::make("h264parse")
        .name("h264parse")
        .build()?;

    let aacparse = gstreamer::ElementFactory::make("aacparse")
        .name("aacparse")
        .build()?;

    let flvmux = gstreamer::ElementFactory::make("flvmux")
        .name("flvmux")
        .property("streamable", true)
        .build()?;

    let rtmp_out = gstreamer::ElementFactory::make("rtmpsink")
        .name("rtmp_out")
        .build()?;

    let pipeline = gstreamer::Pipeline::with_name("test-pipeline");
    pipeline.add_many([
        &srt_source,
        &tsdemux,
        &multiqueue,
        &h264parse,
        &aacparse,
        &flvmux,
        &rtmp_out,
    ])?;

    srt_source.link(&tsdemux)?;

    let mq_audio_pad = multiqueue
        .request_pad_simple("sink_%u")
        .expect("could not get multiqueue pad");
    let mq_video_pad = multiqueue
        .request_pad_simple("sink_%u")
        .expect("could not get multiqueue pad");

    let h264parse_src_pad = h264parse.static_pad("src").unwrap();
    let flv_video_pad = flvmux
        .static_pad("audio")
        .expect("could not get flxmux video pad");
    h264parse_src_pad.link(&flv_video_pad)?;

    let aacparse_src_pad = aacparse.static_pad("src").unwrap();
    let flv_audio_pad = flvmux
        .static_pad("audio")
        .expect("could not get flxmux audio pad");
    aacparse_src_pad.link(&flv_audio_pad);

    flvmux.link(&rtmp_out)?;

    srt_source.connect("caller-connecting", true, |a| {
        println!("caller-connecting : {:?}", a);
        Some(Value::from(true))
        //None
    });

    decoder.connect_pad_added(move |_, src_pad| {
        println!(
            "Received new pad '{:?}' from '{:?}'",
            src_pad.name(),
            src_pad.parent_element().unwrap().name()
        );

        let sink_pad = video_sink
            .static_pad("sink")
            .expect("Failed to get static sink pad from videosink");
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

    pipeline.set_state(gstreamer::State::Playing)?;

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

        let val = srt_source.property_value("stats");

        /*  if let Ok(stats_struct) = val.get::<gstreamer::Structure>() {
            match SrtStatisticsReport::try_from(stats_struct) {
                Ok(stats) => println!("test {:?}",stats),
                Err(e) => println!("err {e}"),
            };
        } else {
            println!("err")
        }*/
    }

    // 8. Clean up
    pipeline.set_state(gstreamer::State::Null)?;
    Ok(())
}
