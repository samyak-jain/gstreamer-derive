#[derive(GStreamer)]
#[link_elements(SRC, URI_DECODE_BIN)]
enum GStreamer {
  #[name = "source"]
  SRC,
  
  URI_DECODE_BIN,
}

#[derive(GStreamer)]
struct CustomData {
  source
}

gstreamer!(
    uridecodebin ! test
)
