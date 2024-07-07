use serde::{Deserialize, Serialize};

/// EventSeries contain information on series of events, i.e. thing that was/is happening continuously for some time.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventSeries {
    /// Number of occurrences in this series up to the last heartbeat time
    pub count: Option<i32>,

    // Time of the last occurrence observed
    // pub last_observed_time: Option<crate::apimachinery::pkg::apis::meta::v1::MicroTime>,
}