pub mod nnue;

use std::ops::{Add, Neg, Sub};

use crate::MAX_DEPTH;


#[derive(Debug, Clone, Copy, Eq)]
pub enum Eval {
    MateIn(u8),
    MatedIn(u8),
    CentiPawn(i32),
}


impl Eval {
    pub const MAX: Eval = Eval::CentiPawn(10_000_000);
    pub const BEST_EVAL: Eval = Eval::MateIn(1);
    pub const NEUTRAL: Eval = Eval::CentiPawn(0);
    pub const WORST_EVAL: Eval = Eval::MatedIn(1);
    pub const MIN: Eval = Eval::CentiPawn(-10_000_000);

    const UNIT: Eval = Eval::CentiPawn(1);

    pub const fn value(self) -> i32 {
        match self {
            Eval::MateIn(x) => 100_000 - x as i32,
            Eval::MatedIn(x) => -100_000 + x as i32,
            Eval::CentiPawn(x) => x,
        }
    }

    pub fn add_ply(self, ply: u8) -> Self {
        match self {
            Eval::MateIn(x) => Eval::MateIn(x.saturating_add(ply)),
            Eval::MatedIn(x) => Eval::MatedIn(x.saturating_add(ply)),
            Eval::CentiPawn(_) => self,
        }
    }

    pub fn sub_ply(self, ply: u8) -> Self {
        match self {
            Eval::MateIn(x) => Eval::MateIn(x.saturating_sub(ply)),
            Eval::MatedIn(x) => Eval::MatedIn(x.saturating_sub(ply)),
            Eval::CentiPawn(_) => self,
        }
    }

    fn normalize(self) -> Self {
        match self {
            Eval::MateIn(_) | Eval::MatedIn(_) => self,
            Eval::CentiPawn(x) => {
                if Self::BEST_EVAL.value() - x < MAX_DEPTH as i32 {
                    Eval::MateIn((Self::BEST_EVAL.value() - x + 1) as u8)
                } else if x - Self::WORST_EVAL.value() < MAX_DEPTH as i32 {
                    Eval::MatedIn((x - Self::WORST_EVAL.value() + 1) as u8)
                } else {
                    self
                }
            }
        }
    }
}

impl Add for Eval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Eval::CentiPawn(self.value() + rhs.value()).normalize()
    }
}

impl Sub for Eval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Eval::CentiPawn(self.value() - rhs.value()).normalize()
    }
}

impl Neg for Eval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Eval::MateIn(x) => Eval::MatedIn(x),
            Eval::MatedIn(x) => Eval::MateIn(x),
            Eval::CentiPawn(x) => Eval::CentiPawn(-x),
        }
    }
}

impl Ord for Eval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Eval {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

#[test]
fn cmp_evals() {
    // To make sure that evals work properly
    assert!(Eval::MAX > Eval::BEST_EVAL);
    assert!(Eval::WORST_EVAL < Eval::NEUTRAL);
    assert!(Eval::NEUTRAL < Eval::BEST_EVAL);
    assert!(Eval::BEST_EVAL > Eval::WORST_EVAL);
    assert!(Eval::BEST_EVAL.max(Eval::NEUTRAL) == Eval::BEST_EVAL);
    assert!(Eval::NEUTRAL.max(Eval::BEST_EVAL) == Eval::BEST_EVAL);
    assert!(Eval::MIN < Eval::WORST_EVAL);
}

#[test]
fn test_normalize() {
    let evals = [
        Eval::BEST_EVAL,
        Eval::WORST_EVAL,
        Eval::MateIn(10),
        Eval::MatedIn(10),
        Eval::CentiPawn(1200),
        Eval::CentiPawn(-1000),
        Eval::CentiPawn(10),
        Eval::CentiPawn(-25),
    ];

    for eval in evals {
        assert_eq!(
            (eval + Eval::NEUTRAL).normalize(),
            eval,
            "{:?} normalized gives {:?}",
            eval,
            (eval + Eval::NEUTRAL).normalize()
        );

        assert_eq!(
            (eval + Eval::UNIT).normalize(),
            eval + Eval::UNIT,
            "{:?} + Eval::UNIT normalized gives {:?}",
            eval,
            (eval + Eval::UNIT).normalize()
        );
    }
}
