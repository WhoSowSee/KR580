use super::messages::SpeedTier;
use crate::platform;

pub(crate) const SLOW_TIER_HZ: u32 = 5;
pub(crate) const MEDIUM_TIER_HZ: u32 = 20;
pub(crate) const HIGH_TIER_FALLBACK_HZ: u32 = 60;
pub(crate) const HIGH_TIER_CEILING_HZ: u32 = 240;
pub(crate) const MAX_TIER_HZ: u32 = 1000;

pub(crate) fn tier_hz(tier: SpeedTier) -> u32 {
    match tier {
        SpeedTier::Slow => SLOW_TIER_HZ,
        SpeedTier::Medium => MEDIUM_TIER_HZ,
        SpeedTier::High => platform::primary_monitor_refresh_hz()
            .unwrap_or(HIGH_TIER_FALLBACK_HZ)
            .clamp(HIGH_TIER_FALLBACK_HZ, HIGH_TIER_CEILING_HZ),
        SpeedTier::Max => MAX_TIER_HZ,
    }
}
