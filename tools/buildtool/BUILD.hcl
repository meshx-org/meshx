midl "com.example" {
  sources = glob("**/*.toml", current)

  #public_deps = [
  #  "sdk/midl/meshx.drm",
  #  "sdk/midl/meshx.media2",
  #  "sdk/midl/meshx.mem",
  #  "fiber/vdso/zx",
  #]
}

component "buildtool_component" {
  deps = [":bin"]
}

meshx_package "buildtool" {
  deps = [
    ":buildtool_component"
  ]
}

/*resource "font" {
  sources = [terminal_font_path]
  outputs = ["data/font.ttf"]
}

component "terminal_component" {
  component_name = "terminal"
  manifest       = "meta/terminal.cml"
  deps           = [":bin"]
}

package "terminal" {
  deps = [
    ":terminal_component",
    ":vsh-terminal",
  ]
}*/