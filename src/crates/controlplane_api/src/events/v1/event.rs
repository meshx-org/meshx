use serde::{Deserialize, Serialize};

/// Event is a report of an event somewhere in the cluster. It generally denotes some state change in the system. Events have a limited retention time and triggers and messages may evolve with time.  Event consumers should not rely on the timing of an event with a given Reason reflecting a consistent underlying trigger, or the continued existence of events with that Reason.  Events should be treated as informative, best-effort, supplemental data.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Event {
    /// action is what action was taken/failed regarding to the regarding object. It is machine-readable. This field cannot be empty for new Events and it can have at most 128 characters.
    pub action: Option<String>,

    /// deprecatedCount is the deprecated field assuring backward compatibility with core.v1 Event type.
    pub deprecated_count: Option<i32>,

    /// deprecatedFirstTimestamp is the deprecated field assuring backward compatibility with core.v1 Event type.
    pub deprecated_first_timestamp: Option<crate::apimachinery::apis::meta::v1::Time>,

    /// deprecatedLastTimestamp is the deprecated field assuring backward compatibility with core.v1 Event type.
    pub deprecated_last_timestamp: Option<crate::apimachinery::apis::meta::v1::Time>,

    /// deprecatedSource is the deprecated field assuring backward compatibility with core.v1 Event type.
    pub deprecated_source: Option<crate::core::v1::EventSource>,

    /// eventTime is the time when this Event was first observed. It is required.
    pub event_time: Option<crate::apimachinery::apis::meta::v1::MicroTime>,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata
    pub metadata: crate::apimachinery::apis::meta::v1::ObjectMeta,

    /// note is a human-readable description of the status of this operation. Maximal length of the note is 1kB, but libraries should be prepared to handle values up to 64kB.
    pub note: Option<String>,

    /// reason is why the action was taken. It is human-readable. This field cannot be empty for new Events and it can have at most 128 characters.
    pub reason: Option<String>,

    /// regarding contains the object this Event is about. In most cases it's an Object reporting controller implements, e.g. ReplicaSetController implements ReplicaSets and this event is emitted because it acts on some changes in a ReplicaSet object.
    pub regarding: Option<crate::core::v1::ObjectReference>,

    /// related is the optional secondary object for more complex actions. E.g. when regarding object triggers a creation or deletion of related object.
    pub related: Option<crate::core::v1::ObjectReference>,

    /// reportingController is the name of the controller that emitted this Event, e.g. `kubernetes.io/kubelet`. This field cannot be empty for new Events.
    pub reporting_controller: Option<String>,

    /// reportingInstance is the ID of the controller instance, e.g. `kubelet-xyzf`. This field cannot be empty for new Events and it can have at most 128 characters.
    pub reporting_instance: Option<String>,

    /// series is data about the Event series this event represents or nil if it's a singleton Event.
    pub series: Option<super::EventSeries>,

    /// type is the type of this event (Normal, Warning), new types could be added in the future. It is machine-readable. This field cannot be empty for new Events.
    pub type_: Option<String>,
}

impl crate::Resource for Event {
    const API_VERSION: &'static str = "events.k8s.io/v1";
    const GROUP: &'static str = "events.k8s.io";
    const KIND: &'static str = "Event";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "events";
    type Scope = crate::NamespaceResourceScope;
}

impl crate::ListableResource for Event {
    const LIST_KIND: &'static str = "EventList";
}

impl crate::Metadata for Event {
    type Ty = crate::apimachinery::apis::meta::v1::ObjectMeta;

    fn metadata(&self) -> &<Self as crate::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut<Self as crate::Metadata>::Ty {
        &mut self.metadata
    }
}