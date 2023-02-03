library {
  name = "example"
}

use {
  name = test
  as   = geo
}

struct Test {
  resource = false

  member test { type = geo.Rect }
  member matrix {
    type = array
    of { type = string }
  }
}

protocol Test {
  discoverable = true

  method do_something {
    doc = "Test event"
    request {}
    response {}
    error {}
  }

  event on_event {
    doc = "Test event"
  }
}