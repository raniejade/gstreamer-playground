use gst::prelude::*;

fn main() {
    // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Could not create source element.");
    let convert = gst::ElementFactory::make("videoconvert", Some("convert"))
        .expect("Could not create convert element.");
    let scale = gst::ElementFactory::make("videoscale", Some("scale"))
        .expect("Could not create scale element");
    let encode = gst::ElementFactory::make("x264enc", Some("encode"))
        .expect("Could not create encode element");
    let mp4mux =
        gst::ElementFactory::make("mp4mux", Some("mux")).expect("Could not create mux element");
    let sink =
        gst::ElementFactory::make("filesink", Some("sink")).expect("Could not create sink element");

    // Build the pipeline
    let uri = "file:///home/raniejade/Downloads/sintel_trailer-480p.webm";

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline
        .add_many(&[&source, &convert, &scale, &encode, &mp4mux, &sink])
        .unwrap();
    convert.link(&scale).expect("Failed to link convert ! scale");
    scale.link_filtered(
        &encode,
        &gst::Caps::builder("video/x-raw")
            .field("height", 320i32)
            .build(),
    ).expect("Failed to set caps filter for scale");

    gst::Element::link_many(&[&encode, &mp4mux, &sink])
        .expect("encode ! mp4mux ! sink could not be linked.");


    // Modify the source's properties
    source.set_property_from_str("uri", uri);
    sink.set_property_from_str("location", "scaled.mp4");

    // Connect the pad-added signal
    source.connect_pad_added(move |src, src_pad| {
        println!("Received new pad {} from {}", src_pad.name(), src.name());

        let sink_pad = convert
            .static_pad("sink")
            .expect("Failed to get static sink pad from convert");
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring.");
            return;
        }

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of caps.");
        let new_pad_type = new_pad_struct.name();

        let is_video = new_pad_type.starts_with("video/x-raw");
        if !is_video {
            println!(
                "It has type {} which is not raw video. Ignoring.",
                new_pad_type
            );
            return;
        }

        let res = src_pad.link(&sink_pad);
        if res.is_err() {
            println!("Type is {} but link failed.", new_pad_type);
        } else {
            println!("Link succeeded (type {}).", new_pad_type);
        }
    });

    // Start playing
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    // Wait until error or EOS
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        }
    }

    // Shutdown pipeline
    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}
