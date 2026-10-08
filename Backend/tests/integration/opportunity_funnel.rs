//! Integration test: opportunity funnel state machine.
//!
//! Verifies the legal state transitions for an opportunity:
//! discovered → qualified → in_conversation → proposed → negotiating → won
//! (with terminal `lost` branches).

// F-AUDIT-51: the workspace lint set denies `clippy::unwrap_used`,
// `expect_used` and `panic` because an `unwrap` on a `Result` can take a
// production service down. In a test binary the opposite holds: panicking IS
// the failure signal, and `unwrap()` is the idiomatic way to assert "this
// fixture must be valid, and if it is not the test must fail". These suites
// were never compiled by any crate before the `[[test]]` targets were added
// in `crates/test-utils/Cargo.toml`, so they never faced the gate.
// The exemption is file-scoped so the production lints stay fully intact.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunnelStage {
    Discovered,
    Qualified,
    InConversation,
    Proposed,
    Negotiating,
    Won,
    Lost,
}

#[derive(Debug)]
struct Opportunity {
    id: String,
    stage: FunnelStage,
    history: Vec<(FunnelStage, FunnelStage)>, // (from, to)
}

impl Opportunity {
    fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            stage: FunnelStage::Discovered,
            history: vec![],
        }
    }

    fn transition(&mut self, target: FunnelStage) -> Result<(), String> {
        let legal = match self.stage {
            FunnelStage::Discovered => &[FunnelStage::Qualified, FunnelStage::Lost][..],
            FunnelStage::Qualified => &[FunnelStage::InConversation, FunnelStage::Lost][..],
            FunnelStage::InConversation => &[FunnelStage::Proposed, FunnelStage::Lost][..],
            FunnelStage::Proposed => &[
                FunnelStage::Negotiating,
                FunnelStage::Won,
                FunnelStage::Lost,
            ][..],
            FunnelStage::Negotiating => &[FunnelStage::Won, FunnelStage::Lost][..],
            FunnelStage::Won | FunnelStage::Lost => &[][..],
        };
        if legal.contains(&target) {
            self.history.push((self.stage, target));
            self.stage = target;
            Ok(())
        } else {
            Err(format!(
                "illegal transition {:?} → {:?}",
                self.stage, target
            ))
        }
    }
}

#[test]
fn happy_path_discovered_to_won() {
    let mut opp = Opportunity::new("opp-1");
    opp.transition(FunnelStage::Qualified).unwrap();
    opp.transition(FunnelStage::InConversation).unwrap();
    opp.transition(FunnelStage::Proposed).unwrap();
    opp.transition(FunnelStage::Negotiating).unwrap();
    opp.transition(FunnelStage::Won).unwrap();
    assert_eq!(opp.stage, FunnelStage::Won);
    assert_eq!(opp.history.len(), 5);
}

#[test]
fn lost_branch_anywhere() {
    let mut opp = Opportunity::new("opp-2");
    opp.transition(FunnelStage::Qualified).unwrap();
    opp.transition(FunnelStage::Lost).unwrap();
    assert_eq!(opp.stage, FunnelStage::Lost);
}

#[test]
fn illegal_skip_denied() {
    let mut opp = Opportunity::new("opp-3");
    // Can't jump from Discovered → Proposed.
    let result = opp.transition(FunnelStage::Proposed);
    assert!(result.is_err());
    assert_eq!(
        opp.stage,
        FunnelStage::Discovered,
        "stage must remain unchanged on illegal transition"
    );
}

#[test]
fn terminal_state_blocks_further() {
    let mut opp = Opportunity::new("opp-4");
    opp.transition(FunnelStage::Qualified).unwrap();
    opp.transition(FunnelStage::Lost).unwrap();
    // Won is illegal from Lost.
    let result = opp.transition(FunnelStage::Won);
    assert!(result.is_err());
}

#[test]
fn history_records_each_transition() {
    let mut opp = Opportunity::new("opp-5");
    opp.transition(FunnelStage::Qualified).unwrap();
    opp.transition(FunnelStage::InConversation).unwrap();
    opp.transition(FunnelStage::Proposed).unwrap();
    assert_eq!(
        opp.history,
        vec![
            (FunnelStage::Discovered, FunnelStage::Qualified),
            (FunnelStage::Qualified, FunnelStage::InConversation),
            (FunnelStage::InConversation, FunnelStage::Proposed),
        ]
    );
}

#[test]
fn phi_score_threshold_qualifies_opportunities() {
    // phi_score ≥ 0.65 promotes to qualified; below stays at discovered.
    fn should_qualify(phi_score: f32) -> bool {
        phi_score >= 0.65
    }
    assert!(should_qualify(0.65));
    assert!(should_qualify(0.95));
    assert!(!should_qualify(0.50));
    assert!(!should_qualify(0.0));
}
