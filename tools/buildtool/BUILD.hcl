midl "com.example" {
  sources = glob("**/*.toml", current)

  #public_deps = [
  #  "sdk/midl/meshx.drm",
  #  "sdk/midl/meshx.media2",
  #  "sdk/midl/meshx.mem",
  #  "fiber/vdso/zx",
  #]
}

meshx_component "buildtool_component" {
  manifest = "meta/buildtool.cml"
  deps     = [":bin"]
}

meshx_package "buildtool" {
  deps = [
    ":buildtool_component"
  ]
}

resource "font" {
  sources = [terminal_font_path]
  outputs = ["data/font.ttf"]
}

/*meshx_component "terminal_component" {
  component_name = "terminal"
  manifest       = "meta/terminal.cml"
  deps           = [":bin"]
}

meshx_package "terminal" {
  deps = [
    ":terminal_component",
    ":vsh-terminal",
  ]
}*/