@0x91b9cce5437dcbad;

using Go = import "/go.capnp";

$Go.package("resource_store");
$Go.import("api/resource_store");

#interface ResourceKind  {
#  create @0 (name :Text) -> (rt :ResourceKind );
#  delete @1 (name :Text);
#}

#struct ResourceObject(Spec) {
#  apiVersion @0 :Text;
#  spec @1 :Spec;
#}

#interface ComponentKind extends(ResourceKind) {
#  getResource @0 (name :Text) -> (res :ResourceObject(ComponentSpec));
#  
#  struct ComponentSpec {
#    name @0 :Text;
#    node @1 :Text;
#  }
#}

struct ResourceObject(Spec, Status) {
  apiVersion @0 :Text;
  metadata: @1 :Meta;
  spec @2 :Spec;
  status @3 :Status;
}

interface ResourceKind(Spec, Status) {
  # A generic interface, with non-generic methods.
  getObject @0 () -> (value :ResourceObject(Spec, Status));
  createObject @1 (value :ResourceObject(Spec, Status)) -> ();
}

#interface ResourceStore {
  # A generic interface, with non-generic methods.
#  getKind @0 (kind :Text) -> (value :ResourceKind);
#}

interface ResourceStore {
  getKind @0 [Spec, Status] (kind :Text) -> (resKind :ResourceKind(Spec, Status));
  # A generic method.
}