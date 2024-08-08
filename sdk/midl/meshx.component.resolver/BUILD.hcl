midl "meshx.component.resolver" {
  sources = glob("**/*.midl", current)
  deps    = ["sdk/midl/fx:fx"]
}