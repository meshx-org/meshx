/// Event is a report of an event somewhere in the cluster.  Events have a limited retention time and triggers and messages may evolve with time.  Event consumers should not rely on the timing of an event with a given Reason reflecting a consistent underlying trigger, or the continued existence of events with that Reason.  Events should be treated as informative, best-effort, supplemental data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Event {
    /// What action was taken/failed regarding to the Regarding object.
    pub action: Option<String>,

    /// The number of times this event has occurred.
    pub count: Option<i32>,

    // Time when this Event was first observed.
    // pub event_time: Option<crate::apimachinery::pkg::apis::meta::v1::MicroTime>,

    // The time at which the event was first recorded. (Time of server receipt is in TypeMeta.)
    // pub first_timestamp: Option<crate::apimachinery::pkg::apis::meta::v1::Time>,

    // The object that this event is about.
    pub involved_object: super::ObjectReference,

    // The time at which the most recent occurrence of this event was recorded.
    // pub last_timestamp: Option<crate::apimachinery::pkg::apis::meta::v1::Time>,
    /// A human-readable description of the status of this operation.
    pub message: Option<String>,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata
    pub metadata: crate::apimachinery::apis::meta::v1::ObjectMeta,

    /// This should be a short, machine understandable string that gives the reason for the transition into the object's current status.
    pub reason: Option<String>,

    // Optional secondary object for more complex actions.
    pub related: Option<super::ObjectReference>,
    
    /// Name of the controller that emitted this Event, e.g. `kubernetes.io/kubelet`.
    pub reporting_component: Option<String>,

    /// ID of the controller instance, e.g. `kubelet-xyzf`.
    pub reporting_instance: Option<String>,

    // Data about the Event series this event represents or nil if it's a singleton Event.
    pub series: Option<super::EventSeries>,

    // The component reporting this event. Should be a short machine understandable string.
    pub source: Option<super::EventSource>,
    
    /// Type of this event (Normal, Warning), new types could be added in the future
    pub type_: Option<String>,
}

impl crate::Resource for Event {
    const API_VERSION: &'static str = "v1";
    const GROUP: &'static str = "";
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
