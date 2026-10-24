# Carcinisation
_Convergent plugin library_

This project provides a framework for language-agnostic plugin gRPC service loading in Rust.  
By using protobuf as a common interface format, gRPC + IPC can bridge the FFI gap.

Nothing particularly novel, but it does mean plugins are free to implement services, IPC, and packaging
in their own languages.

The name "[carcinisation](https://en.wikipedia.org/wiki/Carcinisation)" references the happy accident of
multiple species' [convergent evolution](https://en.wikipedia.org/wiki/Convergent_evolution) into crab-like forms 🦀

### Elevator pitch
Multiple languages and frameworks support gRPC client and server generation from protobuf files.  
We can leverage that for language-agnostic IPC.

### Implementation
Rust can generate compiled code from protobuf files and, surprisingly, can dynamically dispatch
at runtime with protobuf reflection.

This project leverages both:
* _Compile time_ guarantees for host <-> plugin IPC  
* _Run time_ dynamic gRPC service registration   

Plugins communicate with the host in JSON on stdin/stdout while exposing dynamically invoked, 
arbitrary services.


## Crates

Each crate's name is loosely parallel to an aspect of evolutionary biology.

### `phenotype` (n): a single aspect of an organism manifested from its genotype

Runtime IPC contract and services.  
i.e. _A single common feature in potentially heterogeneous code_


### `morphology` (n): the overall form and structure of an organism

Packaging logic: package format, manifest format, etc.  
i.e. _The conforming outer shape of plugin packages_


### `homoplasy` (n): correspondence in form or structure due to similar environments

Plugin loader for uniform interaction with modular services.   
i.e. _Loads plugins that may have heterogeneously converged in form_


### `carapace` (n): the upper section of the exoskeleton in crustaceans

Example project using the plugin loader.  
i.e. _Demonstration of heterogeneous code usage_


### Dependency

Conveniently, dependency in the Rust code flows this direction, just like species classification:

```
homoplasy --> depends on --> morphology --> depends on --> phenotype
```   


## Packaging

### Plugin package format

Each package is assumed to hold only a single plugin.  
Plugins are packaged as XZ-compressed tarballs with the following nominal structure:
```text
some_plugin.tar.xz/
                    checksums.txt
                    manifest.json
                    proto/
                            some_proto_files.proto
                    server/
                            some_binary
                            some_libraries.so
```

### Package manifest format

Packages ship with a JSON manifest declaring info about their contents.
The current manifest version (v1) requires the following fields:

| Field              | Acceptable value(s)                 | Notes                  |
|--------------------|-------------------------------------|------------------------|
| `manifest_version` | `1`                                 | Must be `1`            |
| `package_version`  | `1`                                 | Must be `1`            |
| `plugin_name`      | Human-friendly plugin name          |                        |
| `plugin_version`   | SemVer plugin version, e.g. `0.1.0` |                        |
| `proto_dir`        | e.g. `proto`                        | From the above example |
| `execution_dir`    | e.g. `server`                       | From the above example |
| `entry_point`      | e.g. `some_binary`                  | From the above example |