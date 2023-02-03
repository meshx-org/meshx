library {
  name    = "meshx.data.kv"
  version = "2023-01-01"

  use "*" {
    from = "meshx.test@2023-01-01"
  }

  use "Data" {
    from = "meshx.test@2023-01-01"
  }
}

generator client {
  provider = "prisma-client-js"
}

constants {
    # locals can be bare values like:
    wee = local.baz
    # locals can also be set with other variables :
    baz = "Foo is '${var.foo}' but not '${local.wee}'"
}

enum TestEnum {
    annotations {
        value = "meshx.co/test"
    }

    values = ["test"]
}

struct TestStruct {
    tags {
        value = "meshx.co/test"
    }

    field test { 
        type = string
        optional = true 
        default = const.valami

        tags {
            value = "meshx.co/test"
        }
    }
    
    field data { type = string }
}

// this is 
protocol TestProtocol {
    tags {
        value = "meshx.co/test"
    }

    method getStruct {
        tags {
            value = "meshx.co/test"
        }

        request {
            type = struct.TestStruct
        }
        response {
            type    = struct.TestStruct
            error   = struct.TestStruct
        }
    }
}