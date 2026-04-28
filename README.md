# rpcinisation
_Convergent plugin library_

This project provides a framework for language-agnostic plugin service loading in Rust.  
By using protobuf as a common interface format, RPC bridges the FFI gap.

Nothing particularly novel, but it does mean plugins can be in any language.

The name is a play on [carcinisation](https://en.wikipedia.org/wiki/Carcinisation), the happy accident of
multiple species' [convergent evolution](https://en.wikipedia.org/wiki/Convergent_evolution) into crab-like forms 🦀

**Elevator pitch**  
Multiple languages and frameworks support gRPC client and server generation from protobuf files.  

**Implementation**
Rust can generate compiled code from protobuf files or dynamically dispatch calls using runtime protobuf reflection.

This project combines both:
* _Compile-time_ guarantees for host-plugin IPC
* _Dynamic_ gRPC service registration   

Plugins communicate with the host in JSON on stdin/stdout while also exposing dynamically invoked, arbitrary services.


## Crates

Each crate's name is a loosely parallel concept in convergent evolution.

### `homoplasy` (n): correspondence in form or structure, owing to a similar environment  
Plugin loader for uniformly modular service functionality.   
i.e. _Loads plugins that may have heterogenously converged in form_


### `morphology` (n): the form and structure of an organisim  

Plugin domain logic: package and manifest formats, etc.  
i.e. _The types of common features that a plugin would expose_


### `phenotype`  (n): the form and structure of an organism from genotype and environment
Contract and services for runtime plugin IPC.  
i.e. _A single common feature in potentially heterogeneous code_


## Dependency

Dependency flows this direction, like species classification:

```
homoplasy --depends--> morphology --depends--> phenotype
```   
