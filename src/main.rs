use anyhow::{Context, Error};
use gstreamer::{glib::Value, prelude::*, DebugGraphDetails, Structure};

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
        .property("location", "rtmp://172.19.136.2:9001")
        .build()?;

    let pipeline = gstreamer::Pipeline::with_name("test-pipeline");
    pipeline.add_many([&srt_source, &tsdemux, &multiqueue,&h264parse,&aacparse,&flvmux,&rtmp_out])?;

    srt_source.link(&tsdemux)?;
    
    
    let mq_video_src_pad  = multiqueue.request_pad_simple("sink_0").expect("could not get multiqueue pad");
    let mq_video_sink_pad = multiqueue.static_pad("src_0").unwrap();
    let h264_src_pad = h264parse.static_pad("sink").unwrap();
    mq_video_sink_pad.link(&h264_src_pad)?;
    h264parse.link(&flvmux)?;

    let mq_audio_src_pad  = multiqueue.request_pad_simple("sink_1").expect("could not get multiqueue pad");
    let mq_audio_sink_pad = multiqueue.static_pad("src_1").unwrap();
    let aac_src_pad = aacparse.static_pad("sink").unwrap();
    mq_audio_sink_pad.link(&aac_src_pad)?;
    aacparse.link(&flvmux)?;
   

    flvmux.link(&rtmp_out)?;

    srt_source.connect("caller-connecting", true, |a| {
        println!("caller-connecting : {:?}",a);
        Some(Value::from(true))
        //None
    });

    tsdemux.connect_pad_added(move |_, ts_src_pad| {
        println!(
            "Received new pad '{:?}' from '{:?}'",
            ts_src_pad.name(),
            ts_src_pad.parent_element().unwrap().name()
        );

        // Check the new pad's type
        let new_pad_caps = ts_src_pad
            .current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get structure of new pad caps.");

        println!("pad caps : {}",new_pad_caps.to_string());
        println!("pad struct : {}",new_pad_struct.to_string());

        if new_pad_struct.name().starts_with("video/x-h264") {
            println!("Pad is a h264 video pad. Linking...");
            let res = ts_src_pad.link(&mq_video_src_pad);
            if res.is_err() {
                println!("Failed to link pads: {:?}", res);
            }
        } else if new_pad_struct.name().starts_with("audio/mpeg") {
            println!("Pad is an aac audio pad. Linking...");
            let res = ts_src_pad.link(&mq_audio_src_pad);
            if res.is_err() {
                println!("Failed to link pads: {:?}", res);
            }

        
        }
        else {
            println!("Pad is not a raw video pad. Ignoring.");
        }



    });

    println!("testtest");
    
    pipeline.set_state(gstreamer::State::Playing).context("ERROR PLAYING")?;

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;

        let res = pipeline.debug_to_dot_data(DebugGraphDetails::all());
println!("{}",res.as_str());
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

        let val = srt_source.property_value("stats") ;

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
