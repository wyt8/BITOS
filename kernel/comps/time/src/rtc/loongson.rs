// SPDX-License-Identifier: MPL-2.0

use crate::{rtc::Driver, SystemTime};

pub struct RtcLoongson;

impl Driver for RtcLoongson {
    fn try_new() -> Option<Self> {
        Some(Self)
    }

    // TODO: implement read_rtc
    fn read_rtc(&self) -> SystemTime {
        SystemTime {
            year: 2025,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            nanos: 0,
        }
    }
}
