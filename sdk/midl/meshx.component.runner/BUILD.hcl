midl "meshx.component.runner" {
  sources = glob("**/*.midl", current)
  deps    = ["sdk/midl/fx:fx"]
}