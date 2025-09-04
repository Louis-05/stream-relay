# Stream-relay


## Depedencies

`apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
      gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
      gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
      gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev`








gst-launch-1.0 -v videotestsrc ! video/x-raw, height=1080, width=1920 ! videoconvert ! x264enc tune=zerolatency ! video/x-h264, profile=high ! mpegtsmux ! srtsink uri=srt://172.19.128.1:9876?mode=caller

export INPUT_PORT=9876 && export OUTPUT_PORT=9876 && export WEB_PORT=9876 && export PASSPHRASE=mysuperpassphrase && GST_DEBUG=3