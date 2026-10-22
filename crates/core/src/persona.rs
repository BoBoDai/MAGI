//! The three MAGI units and their personalities.

use serde::{Deserialize, Serialize};

/// The three brains of the MAGI system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaId {
    /// MELCHIOR-1: the scientist.
    Melchior,
    /// BALTHASAR-2: the mother.
    Balthasar,
    /// CASPER-3: the woman.
    Casper,
}

impl PersonaId {
    pub const ALL: [Self; 3] = [Self::Melchior, Self::Balthasar, Self::Casper];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Melchior => "melchior",
            Self::Balthasar => "balthasar",
            Self::Casper => "casper",
        }
    }

    /// The EVA-style unit label shown in the console.
    pub const fn unit(self) -> &'static str {
        match self {
            Self::Melchior => "MELCHIOR-1",
            Self::Balthasar => "BALTHASAR-2",
            Self::Casper => "CASPER-3",
        }
    }

    pub const fn label_cn(self) -> &'static str {
        match self {
            Self::Melchior => "科学家",
            Self::Balthasar => "母亲",
            Self::Casper => "女性",
        }
    }

    /// Different sampling temperatures keep the three calls from collapsing
    /// into the same opinion.
    pub const fn temperature(self) -> f64 {
        match self {
            Self::Melchior => 0.2,
            Self::Balthasar => 0.4,
            Self::Casper => 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_snake_case_ids() {
        assert_eq!(
            serde_json::to_string(&PersonaId::Melchior).unwrap(),
            "\"melchior\""
        );
        assert_eq!(
            serde_json::to_string(&PersonaId::Balthasar).unwrap(),
            "\"balthasar\""
        );
        assert_eq!(
            serde_json::to_string(&PersonaId::Casper).unwrap(),
            "\"casper\""
        );
    }

    #[test]
    fn all_is_ordered_for_batch_slots() {
        assert_eq!(
            PersonaId::ALL,
            [PersonaId::Melchior, PersonaId::Balthasar, PersonaId::Casper]
        );
    }
}
