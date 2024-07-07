use std::collections::HashMap;

use etcd_client::Client;

use crate::{apiservice::APIService, services::Service};

#[derive(Clone)]
pub struct AppState {
    pub services: HashMap<String, Service>,
    pub client: Client,
}
