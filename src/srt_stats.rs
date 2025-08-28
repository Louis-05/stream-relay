#[derive(Debug)]
pub struct SrtStatistics {
    pub packets_sent: i64,
    pub packets_sent_lost: i32,
    pub packets_retransmitted: i32,
    pub packet_ack_received: i32,
    pub packet_nack_received: i32,
    pub send_duration_us: u64,
    pub bytes_sent: u64,
    pub bytes_retransmitted: u64,
    pub bytes_sent_dropped: u64,
    pub packets_sent_dropped: i32,
    pub send_rate_mbps: f64,
    pub negotiated_latency_ms: i32,
    pub packets_received: i64,
    pub packets_received_lost: i32,
    pub packets_received_retransmitted: i32,
    pub packets_received_dropped: i32,
    pub packet_ack_sent: i32,
    pub packet_nack_sent: i32,
    pub bytes_received: u64,
    pub bytes_received_lost: u64,
    pub receive_rate_mbps: f64,
    pub bandwidth_mbps: f64,
    pub rtt_ms: f64,
    pub caller_address: Option<String>,
}

// Represents the top-level GstStructure
#[derive(Debug)]
pub struct SrtStatisticsReport {
    pub callers: Vec<SrtStatistics>,
    pub bytes_received_total: u64,
}

impl TryFrom<gstreamer::Structure> for SrtStatisticsReport {
    type Error = anyhow::Error;

    fn try_from(value: gstreamer::Structure) -> Result<Self, Self::Error> {
        let mut callers: Vec<SrtStatistics> = Vec::new();

        if let Ok(caller_array) = value.get::<gstreamer::glib::ValueArray>("callers") {
            for statt in caller_array.iter() {
                let gstruct = statt.get::<gstreamer::Structure>()?;
                callers.push(SrtStatistics::try_from(gstruct)?);
            }
        }

        Ok(SrtStatisticsReport {
            callers,
            bytes_received_total: value.get("bytes-received-total")?,
        })
    }
}

impl TryFrom<gstreamer::Structure> for SrtStatistics {
    type Error = anyhow::Error;

    fn try_from(value: gstreamer::Structure) -> Result<Self, Self::Error> {
        Ok(SrtStatistics {
            packets_sent: value.get("packets-sent")?,
            packets_sent_lost: value.get("packets-sent-lost")?,
            packets_retransmitted: value.get("packets-retransmitted")?,
            packet_ack_received: value.get("packet-ack-received")?,
            packet_nack_received: value.get("packet-nack-received")?,
            send_duration_us: value.get("send-duration-us")?,
            bytes_sent: value.get("bytes-sent")?,
            bytes_retransmitted: value.get("bytes-retransmitted")?,
            bytes_sent_dropped: value.get("bytes-sent-dropped")?,
            packets_sent_dropped: value.get("packets-sent-dropped")?,
            send_rate_mbps: value.get("send-rate-mbps")?,
            negotiated_latency_ms: value.get("negotiated-latency-ms")?,
            packets_received: value.get("packets-received")?,
            packets_received_lost: value.get("packets-received-lost")?,
            packets_received_retransmitted: value.get("packets-received-retransmitted")?,
            packets_received_dropped: value.get("packets-received-dropped")?,
            packet_ack_sent: value.get("packet-ack-sent")?,
            packet_nack_sent: value.get("packet-nack-sent")?,
            bytes_received: value.get("bytes-received")?,
            bytes_received_lost: value.get("bytes-received-lost")?,
            receive_rate_mbps: value.get("receive-rate-mbps")?,
            bandwidth_mbps: value.get("bandwidth-mbps")?,
            rtt_ms: value.get("rtt-ms")?,
            caller_address: value.get("caller-address").ok(),
        })
    }
}
