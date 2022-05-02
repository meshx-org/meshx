# MeshX Workspace Control Tool
Manages various aspects of a running workspace

## Add Repostiory
Add a repositry to the workspace local registry

```bash
$ mxctl package repo add $REPO http://$HOST_ADDRESS:8083/config.json
```

## Resolve Package
Resolves a package by url but no installs it

```bash
$ mxctl package resolve mxpkg://meshx.co/core
```

## Install package
Installs a package (creates all the resources described in it)

```bash
$ mxctl package install mxpkg://meshx.co/core -s $SPACE
```

## Uninstall package
```bash
$ mxctl package uninstall mxpkg://meshx.co/core -s $SPACE
```

## Spawn a component from a package
Resolves a component if not exists locally yet and then spawns an instance

```bash
$ mxctl create instance --componentUrl mxpkg://meshx.co/hello-world#components/hello-world.yml
URL: mxpkg://meshx.co/hello-world#meta/hello-world.yml
Moniker: /test-space:hello-world
Creating component instance...

# or

$ mxctl spawn mxpkg://meshx.co/hello-world#components/hello-world.yml
URL: mxpkg://meshx.co/hello-world#meta/hello-world.yml
Moniker: /test-space:hello-world
Creating component instance...

# or

# Create multiple YAML objects from stdin
$ cat <<EOF | mxctl apply -f -
apiVersion: components/v1
kind: ComponentInstance
metadata:
  name: hello-world
  space: /test-space
spec:
  componentUrl: mxpkg://meshx.co/hello-world#components/hello-world.yml
  args:
    - --text Hello
EOF
URL: mxpkg://meshx.co/hello-world#meta/hello-world.yml
Moniker: /test-space:hello-world
Creating component instance...
```

## Describe a component instance
Get information about a running component instance

```bash
$ mxctl describe instance pkg-resolver
URL: mxpkg://meshx.co/pkg-resolver#components/pkg-resolver.yml
Type: Dynamic component
Component State: Resolved
Execution State: Running
```

## Destroy a component instance

```bash
$ mxctl destroy instance pkg-resolver
```