resource {
  type    = "Kind"
  version = "2023-01-01"

  meta {
    name = ""
  }

  properties {
    library {
      namespace = "meshx.test"
      version   = "2023-01-01"

      use "*" {
        from = "meshx.test@2023-01-01"
        as   = "meshx.test"
      }

      use "Data" {
        from = "meshx.test@2023-01-01"
      }
    }

    struct Error {}

    // Defines something
    protocol Batch {
      method get {
      }

      method set {
        request {
          struct "test" {

          }
        }
      }
    }

    protocol KV {
      discoverable = true

      method get {
        request { type = struct.Data }
        response { type = struct.Data }
      }

      method set {
        request { type = struct.Data }
        response { type = struct.Data }
      }

      method delete {
        request { type = struct.Data }
        response { type = struct.Data }
      }

      method close {
        error { type = struct.Error }
      }

      method batch {
        request { type = type.data }
        response { type = type.data }
      }
    }
  }
}