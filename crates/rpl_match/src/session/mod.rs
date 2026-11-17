mod bindings;
mod collect;
mod config;
mod inner;
mod matching;
mod multi_matched;
mod slot;

pub use bindings::{BindingSnapshot, MetaBindings};
pub use collect::MatchCollectCtxt;
pub use config::{DEFAULT_MAX_SESSION_RESULTS, SessionConfig};
pub use inner::MatchSession;
pub use matching::SessionMatching;
pub use multi_matched::{MultiMatched, OwnedLintMatch, SessionLintTarget};
pub use slot::{
    AdtSlotCandidate, AdtSlotDesc, CrateAdtItem, CrateFnItem, CrateItemIndex, FnMatchContext, FnSlotCandidate,
    FnSlotDesc, MatchSlot, SessionOutcome, SessionResult, SlotAssignment, SlotCandidate, collect_slot_descs,
};
