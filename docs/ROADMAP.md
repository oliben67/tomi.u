# tomi.u Development Roadmap

**Current Status:** Pre-Alpha (Design Phase)

---

## Overview

This document outlines the development timeline for the tomi.u programming language. The roadmap is divided into major version milestones, each with specific goals and deliverables.

---

## Phase 1: Foundation

### v0.1.0 — Lexer & Parser

- [ ] **Lexer Implementation**
  - [ ] Token definitions for all language constructs
  - [ ] Unicode support (UTF-8 source files)
  - [ ] Indentation-based block detection
  - [ ] String interpolation tokenization
  - [ ] Comment handling (single-line `#`, multi-line `###`)

- [ ] **Parser Implementation**
  - [ ] Expression parser (operators, literals, calls)
  - [ ] Statement parser (let, mut, if, for, while, match)
  - [ ] Function and struct declarations
  - [ ] Module system syntax
  - [ ] Decorator syntax (`@entrypoint`, `@constructor`, etc.)

- [ ] **AST Definition**
  - [ ] Complete AST node types
  - [ ] Source location tracking for errors
  - [ ] Pretty-printer for debugging

- [ ] **Testing Infrastructure**
  - [ ] Unit tests for lexer
  - [ ] Parser test suite with edge cases
  - [ ] Error message quality tests

**Deliverable:** Compiler that parses valid tomi.u source files into AST

---

### v0.2.0 — Type System

- [ ] **Type Definitions**
  - [ ] Primitive types (Int8-64, UInt8-64, Float32/64, Bool, Char, String)
  - [ ] Struct types
  - [ ] Enum types (sum types with associated values)
  - [ ] Generic types with type parameters
  - [ ] Type aliases

- [ ] **Type Inference Engine**
  - [ ] Hindley-Milner style inference
  - [ ] Bidirectional type checking
  - [ ] Generic instantiation
  - [ ] Constraint solving

- [ ] **Trait System**
  - [ ] Trait definitions
  - [ ] Trait implementations
  - [ ] Trait bounds on generics
  - [ ] Default method implementations

- [ ] **Type Checking**
  - [ ] Expression type validation
  - [ ] Function signature checking
  - [ ] Generic constraint verification
  - [ ] Exhaustiveness checking for match expressions

**Deliverable:** Full type checking with inference and generics

---

### v0.3.0 — Python Bridge (Bootstrap Runtime)

> **Why Early?** The Python bridge enables running tomi.u code immediately after parsing and type checking, using Python 3.14 as the execution backend. This allows early testing, prototyping, and iterative development before the native runtime is complete.

- [ ] **Python Integration Core**
  - [ ] CPython 3.14 embedding
  - [ ] GIL management
  - [ ] Object lifecycle bridging

- [ ] **Code Generation to Python**
  - [ ] AST-to-Python transpilation
  - [ ] Type mapping to Python equivalents
  - [ ] Runtime library in Python

- [ ] **Type Conversions**
  - [ ] Primitive type marshaling
  - [ ] Collection conversions
  - [ ] Custom type mapping

- [ ] **Bidirectional Interop**
  - [ ] tomi.u calling Python modules
  - [ ] `@python.export` decorator
  - [ ] Callback support
  - [ ] Error propagation

**Deliverable:** Working tomi.u programs via Python backend; full Python 3.14 interoperability

---

### v0.4.0 — Ownership & Memory

- [ ] **Borrow Checker**
  - [ ] Ownership tracking
  - [ ] Move semantics implementation
  - [ ] Immutable borrow analysis
  - [ ] Mutable borrow analysis
  - [ ] Borrow conflict detection

- [ ] **Lifetime Analysis**
  - [ ] Lifetime parameter syntax
  - [ ] Lifetime inference (elision rules)
  - [ ] Lifetime constraint solving
  - [ ] Dangling reference prevention

- [ ] **Drop Semantics**
  - [ ] `@destructor` method invocation
  - [ ] Drop order determination
  - [ ] RAII pattern support

- [ ] **Code Generation (Initial)**
  - [ ] LLVM IR generation
  - [ ] Basic optimizations
  - [ ] Memory layout decisions

**Deliverable:** Memory-safe compilation with borrow checker

---

### v0.5.0 — Async Runtime

- [ ] **Async/Await Core**
  - [ ] Future trait definition
  - [ ] Async function transformation
  - [ ] Await point insertion
  - [ ] State machine generation

- [ ] **Runtime Executor**
  - [ ] Multi-threaded runtime
  - [ ] Work-stealing scheduler
  - [ ] I/O event loop integration
  - [ ] Timer support

- [ ] **Structured Concurrency**
  - [ ] Scope-based task management
  - [ ] Cancellation propagation
  - [ ] Task hierarchy tracking

- [ ] **Synchronization Primitives**
  - [ ] Mutex implementation
  - [ ] RwLock implementation
  - [ ] Channel (mpsc, mpmc)
  - [ ] Semaphore

**Deliverable:** Fully functional async runtime with structured concurrency

---

## Phase 2: Advanced Features

### v0.6.0 — TQL Query Engine

- [ ] **Collection Queries**
  - [ ] `from`/`where`/`select` syntax
  - [ ] `order by`, `take`, `skip`
  - [ ] `group by` with aggregations
  - [ ] Join operations

- [ ] **Query Optimization**
  - [ ] Lazy evaluation
  - [ ] Predicate pushdown
  - [ ] Query plan caching

- [ ] **Graph Query Support**
  - [ ] Node and edge definitions
  - [ ] Pattern matching syntax
  - [ ] Path queries
  - [ ] Graph mutations

- [ ] **Query Compilation**
  - [ ] Type-safe query generation
  - [ ] Query interpolation
  - [ ] Async query execution

**Deliverable:** Integrated SQL/Cypher-inspired query language

---

### v0.7.0 — Actor System

- [ ] **Actor Definitions**
  - [ ] Actor syntax and state isolation
  - [ ] Message type definitions
  - [ ] Message handlers

- [ ] **Actor Runtime**
  - [ ] Actor spawning and lifecycle
  - [ ] Mailbox implementation
  - [ ] Ask/Tell patterns
  - [ ] Actor selection/discovery

- [ ] **Supervision**
  - [ ] Supervisor strategies (OneForOne, AllForOne)
  - [ ] Failure handling
  - [ ] Restart policies

- [ ] **Distribution (Foundation)**
  - [ ] Serialization framework
  - [ ] Remote actor references
  - [ ] Cluster membership basics

**Deliverable:** Complete actor model implementation

---

### v0.8.0 — Aspects & Reflection

- [ ] **Aspect-Oriented Programming**
  - [ ] Aspect definitions
  - [ ] Pointcut expressions
  - [ ] Before/After/Around advice
  - [ ] Compile-time weaving

- [ ] **Built-in Aspects**
  - [ ] `@Log`, `@Timed`, `@Cached`
  - [ ] `@Retry`, `@CircuitBreaker`
  - [ ] `@RateLimit`, `@Transaction`

- [ ] **Reflection API**
  - [ ] Type introspection
  - [ ] Method introspection
  - [ ] Dynamic invocation
  - [ ] Attribute access

- [ ] **Runtime Modification**
  - [ ] Dynamic aspect application
  - [ ] Proxy generation
  - [ ] Behavior interception

**Deliverable:** Full AOP support with reflection capabilities

---

## Phase 3: Production Ready

### v0.9.0 — Standard Library

- [ ] **Core Modules**
  - [ ] `std.io` — I/O operations
  - [ ] `std.fs` — File system
  - [ ] `std.net` — Networking (TCP, UDP, HTTP)
  - [ ] `std.collections` — Data structures

- [ ] **Data Processing**
  - [ ] `std.json` — JSON parsing/serialization
  - [ ] `std.xml` — XML support
  - [ ] `std.csv` — CSV handling
  - [ ] `std.regex` — Regular expressions

- [ ] **Utilities**
  - [ ] `std.time` — Date/time handling
  - [ ] `std.math` — Mathematical functions
  - [ ] `std.crypto` — Cryptographic primitives
  - [ ] `std.random` — Random number generation

- [ ] **System Integration**
  - [ ] `std.env` — Environment variables
  - [ ] `std.process` — Process management
  - [ ] `std.os` — OS-specific APIs

**Deliverable:** Comprehensive standard library

---

### v0.10.0 — Tooling & Polish

- [ ] **CLI Tool (`tomi`)**
  - [ ] `tomi new` — Project scaffolding
  - [ ] `tomi build` — Compilation
  - [ ] `tomi run` — Execute programs
  - [ ] `tomi test` — Test runner
  - [ ] `tomi fmt` — Code formatter
  - [ ] `tomi lint` — Linter

- [ ] **Package Manager**
  - [ ] `tomi.toml` configuration
  - [ ] Dependency resolution
  - [ ] Version constraints
  - [ ] Package registry

- [ ] **IDE Support**
  - [ ] Language Server Protocol (LSP)
  - [ ] VS Code extension
  - [ ] Syntax highlighting
  - [ ] Code completion
  - [ ] Go-to-definition

- [ ] **Documentation**
  - [ ] `tomi doc` — Documentation generator
  - [ ] API reference generation
  - [ ] Example extraction

**Deliverable:** Complete development toolchain

---

### v1.0.0 — Production Release

- [ ] **Stability**
  - [ ] API stabilization (no breaking changes in 1.x)
  - [ ] Performance benchmarks
  - [ ] Security audit
  - [ ] Memory safety verification

- [ ] **Documentation**
  - [ ] Language reference manual
  - [ ] Standard library documentation
  - [ ] Tutorial series
  - [ ] Migration guides

- [ ] **Ecosystem**
  - [ ] Package registry launch
  - [ ] Community guidelines
  - [ ] Contribution process

- [ ] **Platform Support**
  - [ ] Linux (x86_64, ARM64)
  - [ ] macOS (x86_64, ARM64)
  - [ ] Windows (x86_64)

**Deliverable:** Production-ready tomi.u 1.0.0

---

## Phase 4: Maintenance & Evolution

### v1.x.x — Maintenance Releases

- [ ] Bug fixes and security patches
- [ ] Performance improvements
- [ ] Python bridge maintenance (3.14 compatibility)
- [ ] Ecosystem growth support
- [ ] Minor feature additions (backwards compatible)

### v2.0.0 — Future Vision

- [ ] **Native Compilation**
  - [ ] Remove Python bridge requirement
  - [ ] Native code generation for all platforms
  - [ ] Ahead-of-time compilation

- [ ] **Advanced Features**
  - [ ] Effect system
  - [ ] Dependent types (experimental)
  - [ ] Compile-time computation (const generics)

- [ ] **Extended Platforms**
  - [ ] WebAssembly target
  - [ ] Embedded systems support
  - [ ] Mobile platforms (iOS, Android)

---

## Resource Requirements

### Team Composition (Estimated)

| Phase | Engineers | Duration | Focus Areas |
|-------|-----------|----------|-------------|
| Foundation | 3-4 | 12 months | Compiler core, type system |
| Advanced | 5-6 | 12 months | Runtime, TQL, actors |
| Production | 6-8 | 9 months | Std lib, tooling, polish |
| Maintenance | 3-4 | Ongoing | Support, ecosystem |

### Infrastructure

- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Package registry hosting
- [ ] Documentation hosting
- [ ] Community forum/Discord

---

## Risk Factors

| Risk | Mitigation |
|------|------------|
| Borrow checker complexity | Start with simpler subset; iterate |
| Python bridge performance | Optimize hot paths; provide pure tomi.u alternatives |
| TQL query optimization | Build on established query engine research |
| Ecosystem adoption | Provide excellent tooling; Python interop as bridge |

---

## Success Metrics

### v1.0.0 Launch Goals

- [ ] Compiler passes all language specification tests
- [ ] Standard library covers 90% of common use cases
- [ ] Build times < 5 seconds for medium projects
- [ ] Zero known memory safety bugs
- [ ] Documentation coverage > 95%
- [ ] At least 50 community packages in registry

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on participating in tomi.u development.

---

*This roadmap is subject to change based on community feedback and technical discoveries.*
