# Carcinisation
_Convergent plugin library_

This project provides a framework for language-agnostic plugin gRPC service loading in Rust.  
By using protobuf as a common interface format, gRPC + IPC can bridge the FFI gap.

Nothing particularly novel, but it does mean plugins are free to implement services, IPC, and packaging
in their own languages.

The name "[carcinisation](https://en.wikipedia.org/wiki/Carcinisation)" references the happy accident of
multiple species' [convergent evolution](https://en.wikipedia.org/wiki/Convergent_evolution) into crab-like forms 🦀

**Elevator pitch**  
Multiple languages and frameworks support gRPC client and server generation from protobuf files.  

**Implementation**  
Rust can also generate compiled code from protobuf files and dynamically dispatch at runtime with 
protobuf reflection.

This project combines both:
* _Compile time_ guarantees for host <-> plugin IPC  
* _Run time_ dynamic gRPC service registration   

Plugins communicate with the host in JSON on stdin/stdout while exposing dynamically invoked, 
arbitrary services.


## Crates

Each crate's name is loosely parallel to an aspect of evolutionary biology.

### `carapace` (n): the upper section of the exoskeleton in crustaceans

Example project using the plugin loader.
i.e. _Demonstration of heterogeneous code usage_

### `homoplasy` (n): correspondence in form or structure, owing to a similar environment  

Plugin loader for uniformly modular service functionality.   
i.e. _Loads plugins that may have heterogeneously converged in form_


### `morphology` (n): the form and structure of an organism

Packaging logic: package format, manifest format, etc.  
i.e. _The conforming outer shape of plugin packages_


### `phenotype`  (n): the form and structure of an organism from genotype and environment

Runtime IPC contract and services.  
i.e. _A single common feature in potentially heterogeneous code_


## Dependency

Dependency flows this direction, like species classification:

```
homoplasy --depends--> morphology --depends--> phenotype
```   
