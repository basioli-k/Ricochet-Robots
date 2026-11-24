//! Robots and the targets they chase.

/// The four robots; each has one colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RobotColor {
    Red,
    Blue,
    Green,
    Yellow,
}

impl RobotColor {
    pub const ALL: [Self; 4] = [Self::Red, Self::Blue, Self::Green, Self::Yellow];

    /// Stable index in `0..4`, matching the order of [`Self::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// The symbol printed on a coloured target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Symbol {
    Star,
    Planet,
    Gear,
    Moon,
}

impl Symbol {
    pub const ALL: [Self; 4] = [Self::Star, Self::Planet, Self::Gear, Self::Moon];
}

/// A target cell. The game has one of each colour/symbol pair plus one vortex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    /// Reached only by the robot of the same colour.
    Colored(RobotColor, Symbol),
    /// Reached by any robot.
    Vortex,
}

impl Target {
    /// Whether `robot` stopping on this target completes it.
    #[must_use]
    pub fn is_reached_by(self, robot: RobotColor) -> bool {
        match self {
            Self::Colored(color, _) => color == robot,
            Self::Vortex => true,
        }
    }

    /// The 17 targets of a full game: 16 coloured plus the vortex.
    pub fn all() -> impl Iterator<Item = Self> {
        RobotColor::ALL
            .into_iter()
            .flat_map(|color| {
                Symbol::ALL
                    .into_iter()
                    .map(move |symbol| Self::Colored(color, symbol))
            })
            .chain(std::iter::once(Self::Vortex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn colored_target_needs_the_matching_robot() {
        let target = Target::Colored(RobotColor::Red, Symbol::Moon);
        assert!(target.is_reached_by(RobotColor::Red));
        assert!(!target.is_reached_by(RobotColor::Blue));
    }

    #[test]
    fn vortex_is_reached_by_any_robot() {
        assert!(
            RobotColor::ALL
                .into_iter()
                .all(|r| Target::Vortex.is_reached_by(r))
        );
    }

    #[test]
    fn a_game_has_seventeen_distinct_targets() {
        let targets: HashSet<_> = Target::all().collect();
        assert_eq!(targets.len(), 17);
        assert!(targets.contains(&Target::Vortex));
    }

    #[test]
    fn robot_indices_follow_all() {
        for (i, robot) in RobotColor::ALL.into_iter().enumerate() {
            assert_eq!(robot.index(), i);
        }
    }
}
