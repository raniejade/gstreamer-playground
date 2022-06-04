use gst::prelude::*;

fn main() {
   // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Could not create source element.");
    let convert = gst::ElementFactory::make("videoconvert", Some("convert"))
        .expect("Could not create convert element.");
    let sink = gst::ElementFactory::make("autovideosink", Some("sink"))
        .expect("Could not create sink element");

    // Build the pipeline
    let uri =
        "file:///home/raniejade/Downloads/sintel_trailer-480p.webm";
    // let pipeline = gst::parse_launch(&format!("playbin uri={}", uri)).unwrap();

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline.add_many(&[&source, &convert, &sink]).unwrap();
    gst::Element::link_many(&[&convert, &sink]).expect("Elements could not be linked.");

    // Modify the source's properties
    source.set_property_from_str("uri", uri);

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
