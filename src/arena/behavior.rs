use crate::components::AiBehaviorState;

/// Which behavior upgrades are installed on an actor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InstalledBehaviorSet {
    pub patrol: bool,
    pub guard: bool,
    pub aggressive: bool,
    pub flee: bool,
    pub scavenge: bool,
    pub follow_leader: bool,
}

/// Current situation signals used by behavior arbitration.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BehaviorSignals {
    /// Health ratio, usually `current / max`.
    pub health_ratio: f32,
    pub threat_visible: bool,
    pub loot_detected: bool,
    pub guard_zone: bool,
    pub leader_nearby: bool,
    pub has_patrol_waypoints: bool,
}

/// Deterministically chooses the active behavior state.
///
/// Priority order matches the arena roadmap:
///
/// 1. Flee
/// 2. Aggressive
/// 3. Scavenge
/// 4. FollowLeader
/// 5. Guard
/// 6. Patrol
/// 7. Passive
pub fn arbitrate_behavior(
    installed: &InstalledBehaviorSet,
    signals: &BehaviorSignals,
) -> AiBehaviorState {
    if installed.flee && signals.health_ratio <= 0.25 && signals.threat_visible {
        return AiBehaviorState::Flee;
    }

    if installed.aggressive && signals.threat_visible {
        return AiBehaviorState::Aggressive;
    }

    if installed.scavenge && signals.loot_detected && !signals.threat_visible {
        return AiBehaviorState::Scavenge;
    }

    if installed.follow_leader && signals.leader_nearby && !signals.threat_visible {
        return AiBehaviorState::FollowLeader;
    }

    if installed.guard && signals.guard_zone && !signals.threat_visible {
        return AiBehaviorState::Guard;
    }

    if installed.patrol && signals.has_patrol_waypoints {
        return AiBehaviorState::Patrol;
    }

    AiBehaviorState::Passive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flee_overrides_aggressive_at_low_health() {
        let installed = InstalledBehaviorSet {
            flee: true,
            aggressive: true,
            ..Default::default()
        };
        let signals = BehaviorSignals {
            health_ratio: 0.1,
            threat_visible: true,
            ..Default::default()
        };

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::Flee
        );
    }

    #[test]
    fn aggressive_overrides_scavenge_when_threat_is_visible() {
        let installed = InstalledBehaviorSet {
            aggressive: true,
            scavenge: true,
            ..Default::default()
        };
        let signals = BehaviorSignals {
            threat_visible: true,
            loot_detected: true,
            ..Default::default()
        };

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::Aggressive
        );
    }

    #[test]
    fn scavenge_runs_without_threat() {
        let installed = InstalledBehaviorSet {
            scavenge: true,
            ..Default::default()
        };
        let signals = BehaviorSignals {
            loot_detected: true,
            threat_visible: false,
            ..Default::default()
        };

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::Scavenge
        );
    }

    #[test]
    fn follow_leader_precedes_guard() {
        let installed = InstalledBehaviorSet {
            follow_leader: true,
            guard: true,
            ..Default::default()
        };
        let signals = BehaviorSignals {
            leader_nearby: true,
            guard_zone: true,
            ..Default::default()
        };

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::FollowLeader
        );
    }

    #[test]
    fn guard_precedes_patrol() {
        let installed = InstalledBehaviorSet {
            guard: true,
            patrol: true,
            ..Default::default()
        };
        let signals = BehaviorSignals {
            guard_zone: true,
            has_patrol_waypoints: true,
            ..Default::default()
        };

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::Guard
        );
    }

    #[test]
    fn patrol_is_the_fallback_behavior() {
        let installed = InstalledBehaviorSet {
            patrol: true,
            ..Default::default()
        };
        let signals = BehaviorSignals {
            has_patrol_waypoints: true,
            ..Default::default()
        };

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::Patrol
        );
    }

    #[test]
    fn passive_when_no_behavior_is_available() {
        let installed = InstalledBehaviorSet::default();
        let signals = BehaviorSignals::default();

        assert_eq!(
            arbitrate_behavior(&installed, &signals),
            AiBehaviorState::Passive
        );
    }
}
