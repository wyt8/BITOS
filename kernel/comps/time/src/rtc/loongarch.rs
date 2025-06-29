use crate::{rtc::Driver, SystemTime};

pub struct RtcLoongarch;

impl Driver for RtcLoongarch {
    fn try_new() -> Option<Self> {
        Some(Self)
    }

    fn read_rtc(&self) -> SystemTime {
        SystemTime {
            year: 2023,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            nanos: 0,
        }
    }
}
