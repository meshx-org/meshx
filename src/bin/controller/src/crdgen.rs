use controlplane_client::core::CustomResourceExt;
fn main() {
    print!("{}", serde_yml::to_string(&controller::Document::crd()).unwrap())
}