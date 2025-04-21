use solana_program::{
    clock::Clock,
    rent::Rent
};
use chrono;


pub struct ClusterSettings {
    // ++++++ Rent +++++
    pub lamports_per_byte_year: u64,
    pub exemption_threshold: f64,
    pub burn_percent: u8,
    // ++++++ Clock +++++
    pub slot: u64,
    pub epoch_start_timestamp: i64,
    pub epoch: u64,
    pub leader_schedule_epoch: u64,
    pub unix_timestamp: i64,
}

impl ClusterSettings {
    pub fn new() -> Self {
        let time = chrono::Utc::now().timestamp();
        ClusterSettings {
            lamports_per_byte_year: 1_000_000_000 / 100 * 365 / (1024 * 1024),
            exemption_threshold: 2.0,
            burn_percent: 50,
            slot: 1,
            epoch_start_timestamp: time,
            epoch: 1,
            leader_schedule_epoch: 0,
            unix_timestamp: time,
        }
    }

    pub fn as_clock(&self) -> Clock {
        Clock {
            slot: self.slot,
            epoch_start_timestamp: self.epoch_start_timestamp,
            epoch: self.epoch,
            leader_schedule_epoch: self.leader_schedule_epoch,
            unix_timestamp: self.unix_timestamp,
        }
    }

    pub fn as_rent(&self) -> Rent {
        Rent {
            lamports_per_byte_year: self.lamports_per_byte_year,
            exemption_threshold: self.exemption_threshold,
            burn_percent: self.burn_percent,
        }
    }
}