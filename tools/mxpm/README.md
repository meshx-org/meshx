# MeshX Packager Tool

# Init a new repository

```bash
$ mxpm initrepo --repo $REPO
```

### Serve packages

This command will start a local package server

```bash
$ mxpm serve --repo $REPO
```

### Build packages

This command will build a package

```bash
$ mxpm build
```

## Publish package
Publishes a package to the defined repository

```bash
$ mxpm publish -a -r $REPO -f $PACKAGE_ARCHIVE
```