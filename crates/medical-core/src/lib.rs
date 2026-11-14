//! Medical monitoring logic for Nurbit.
//!
//! Rule for this crate: **pure functions only**. Data comes in as arguments and
//! results go out as return values; no database, files, network or clock reads.
//! That makes every rule easy to test and lets the same code run on the
//! astronaut's computer offline and on the server after synchronization.
//!
//! Nothing here is clinically validated. Thresholds added later must carry a
//! `basis` (a source id or "prototype assumption").

pub mod assistant;
pub mod baseline;
pub mod checkin;
pub mod daily_check;
pub mod followup;
pub mod risk_catalog;
pub mod review;
pub mod risk_eval;
