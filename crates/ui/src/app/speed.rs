use super::messages::SpeedTier;

pub(crate) const SLOW_TIER_HZ: u32 = 5;
pub(crate) const MEDIUM_TIER_HZ: u32 = 20;
pub(crate) const HIGH_TIER_HZ: u32 = 120;
pub(crate) const MAX_TIER_HZ: u32 = 1000;

pub(crate) fn tier_hz(tier: SpeedTier) -> u32 {
    match tier {
        SpeedTier::Slow => SLOW_TIER_HZ,
        SpeedTier::Medium => MEDIUM_TIER_HZ,
        SpeedTier::High => HIGH_TIER_HZ,
        SpeedTier::Max => MAX_TIER_HZ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_tiers_keep_legacy_distinct_rates() {
        assert_eq!(tier_hz(SpeedTier::Slow), 5);
        assert_eq!(tier_hz(SpeedTier::Medium), 20);
        assert_eq!(tier_hz(SpeedTier::High), 120);
        assert_eq!(tier_hz(SpeedTier::Max), 1000);
    }
}
