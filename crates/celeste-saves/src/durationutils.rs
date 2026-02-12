pub trait DurationUtils {
    fn formatted(&self) -> String;
}

impl DurationUtils for std::time::Duration {
    fn formatted(&self) -> String {
        let mut pieces = String::new();

        let millis_left = self.as_millis();
        let millis = millis_left % 1000;
        let seconds_left = millis_left / 1000;
        let seconds = seconds_left % 60;
        let minutes_left = seconds_left / 60;
        let minutes = minutes_left % 60;
        let hours = minutes_left / 60;

        if !pieces.is_empty() {
            pieces.push_str(&format!("{:>02}h", hours));
        } else if hours > 0 {
            pieces.push_str(&format!("{:>2}h", hours));
        }

        if !pieces.is_empty() {
            pieces.push_str(&format!("{:>02}m", minutes));
        } else if minutes > 0 {
            pieces.push_str(&format!("{:>2}m", minutes));
        }

        if !pieces.is_empty() {
            pieces.push_str(&format!("{:>02}", seconds));
        } else if seconds > 0 {
            pieces.push_str(&format!("{:>2}", seconds));
        }

        if !pieces.is_empty() || millis > 0 {
            pieces.push_str(&format!(".{:>03}s", millis));
        } else {
            pieces.push_str("0 ");
        }

        format!("{:>13}", pieces)
    }
}
