use fiber_rust as fx;
use fx::{AsHandleRef, HandleBased, Koid};
//use moniker::Moniker;
use std::{collections::HashMap, rc::Rc};

/// [`InstanceRegistry`] maintains mapping from [`InstanceToken`] KOIDs to the
/// moniker of those component instances.
pub struct InstanceRegistry {
    //koid_to_moniker: HashMap<Koid, Moniker>,
}

impl InstanceRegistry {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            //koid_to_moniker: HashMap::new(),
        })
    }
}
