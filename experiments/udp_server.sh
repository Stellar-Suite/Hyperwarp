GST_DEBUG_DUMP_DOT_DIR=/tmp/gst-debug gst-launch-1.0 ximagesrc ! videoconvert ! videoflip ! videoconvert ! video/x-raw,format=NV12 ! tee ! queue ! nvh264enc ! video/x-h264, stream-format=byte-stream, alignment=au ! h264parse ! rtph264pay pt=96 config-interval=1 ! tee ! queue ! udpsink host=127.0.0.1 port=4321