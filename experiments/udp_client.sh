gst-launch-1.0 udpsrc udp://127.0.0.1:4321 ! "application/x-rtp,media=video,encoding-name=H264,payload=96" ! rtph264depay ! video/x-h264 ! queue ! h264parse ! queue ! nvh264dec ! videoconvert ! autovideosink