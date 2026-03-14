# tomi.u Language Specification

**Version:** 1.0.0  
**Status:** Draft

---

## Overview

**tomi.u** is a modern, statically-typed programming language that combines Python's clean, readable syntax with C#'s strict type system. It features ownership-based memory management (no garbage collector), first-class support for asynchronous programming, powerful pattern matching, and an integrated query language for objects and graphs.

---

## Table of Contents

1. [Modern Language Features](#modern-language-features)
2. [Design Principles](#design-principles)
3. [Basic Syntax](#basic-syntax)
4. [Type System](#type-system)
5. [Ownership and Memory Management](#ownership-and-memory-management)
6. [Pattern Matching](#pattern-matching)
7. [Asynchronous Programming](#asynchronous-programming)
8. [Actor Model](#actor-model)
9. [Aspect-Oriented Programming](#aspect-oriented-programming)
10. [Reflection and Runtime Modification](#reflection-and-runtime-modification)
11. [Integrated Query Language (TQL)](#integrated-query-language-tql)
12. [Standard Library Overview](#standard-library-overview)
13. [Python Interoperability](#python-interoperability)
14. [Error Handling](#error-handling)

---

## Modern Language Features

tomi.u incorporates the best features from modern programming languages, providing a comprehensive toolkit for building safe, efficient, and maintainable software.

### Feature Overview

| Feature | tomi.u Implementation |
|---------|----------------------|
| **Memory Safety** | Borrow Checker with ownership rules prevents buffer overflows and dangling pointers at compile-time |
| **Type Inference** | Static inference (`let x = 5`) combines safety of static typing with brevity of dynamic languages |
| **Null Safety** | `Option[T]` types with compile-time enforcement eliminate null pointer exceptions |
| **Native Concurrency** | Async/Await, Actor Model, and Structured Concurrency built into the language |
| **Pattern Matching** | Advanced destructuring with guards, or-patterns, and exhaustiveness checking |
| **Zero-Cost Abstractions** | High-level features compile to optimal machine code with no runtime overhead |
| **Integrated Tooling** | Unified package manager, formatter, linter, and test runner (`tomi` CLI) |
| **Immutability by Default** | Variables are immutable (`let`) unless explicitly declared mutable (`mut`) |
| **Interoperability** | Native Python bridge; C/Rust FFI support for legacy codebases |

### Concurrency Models in tomi.u

Modern languages focus on making multi-core programming safer and easier without "callback hell" or complex locking mechanisms. tomi.u provides multiple concurrency paradigms:

#### Async/Await (The Standard)

Used by JavaScript, Rust, C#, and Python—tomi.u adopts this pattern as its primary async model. Code looks synchronous while being non-blocking:

```tomi.u
async def fetch_user_data(id: UserId) -> Result[UserData, Error]:
    # The program "waits" without freezing the thread
    let profile = await database.get_profile(id)
    let preferences = await cache.get_preferences(id)
    
    Ok(UserData { profile, preferences })
```

#### Actor Model with Message Passing

Inspired by Erlang and Akka, tomi.u's actor system follows the mantra: *"Do not communicate by sharing memory; instead, share memory by communicating."*

```tomi.u
actor DataProcessor:
    state:
        buffer: List[Data]
    
    on Process(data: Data):
        self.buffer.push(data)
    
    on Flush -> List[Data]:
        let result = self.buffer.clone()
        self.buffer.clear()
        result

# Actors communicate via messages, never shared state
let processor = spawn DataProcessor.create()
processor.send(Process(data))  # Fire and forget
let results = await processor.ask(Flush)  # Request-response
```

#### Structured Concurrency

Inspired by Swift and Kotlin, tomi.u ensures asynchronous tasks have a clear hierarchy. If a parent task is cancelled, all child tasks are automatically cleaned up—preventing "zombie" processes:

```tomi.u
async def process_batch(items: List[Item]) -> Result[List[Output], Error]:
    # All spawned tasks are children of this scope
    scope |s|:
        for item in items:
            s.spawn(async || process_item(item))
    # When scope exits:
    # - All tasks complete (or are cancelled if scope fails)
    # - No orphaned tasks possible
```

### Memory Management Philosophy

The goal is to achieve C++ speed without manual risk of `malloc` or `free`. tomi.u uses ownership-based memory management:

#### The Borrow Checker

A revolution in systems programming, enforcing rules at compile-time:

- Each piece of data has exactly **one owner**
- Data can be **borrowed** (referenced) without transferring ownership
- Only **one mutable borrow** OR **multiple immutable borrows** at a time
- No data races or memory leaks—all without a Garbage Collector

```tomi.u
def process(data: &String) -> Unit:     # Immutable borrow
    io.println(data)

def modify(data: &mut String) -> Unit:  # Mutable borrow
    data.push_str(" modified")

let owned = String.from("hello")       # owned has ownership
process(&owned)                         # Borrow for reading
modify(&mut owned)                      # Borrow for writing
# owned still valid here
```

#### Ownership & Lifetimes

tomi.u distinguishes between "owning" data and "borrowing" it, making intent clear to the compiler:

```tomi.u
# Explicit lifetime: returned reference lives as long as inputs
def longest['a](x: &'a String, y: &'a String) -> &'a String:
    if x.len() > y.len(): x else: y

# Ownership transfer (move semantics)
let s1 = String.from("hello")
let s2 = s1              # s1 moved to s2
# s1 no longer valid—compile error if used
```

#### Smart Pointers for Shared Ownership

When single ownership isn't sufficient:

```tomi.u
# Reference Counted (single-threaded)
let shared: Rc[Data] = Rc.create(data)
let clone = shared.clone()  # Increment reference count

# Atomic Reference Counted (thread-safe)
let atomic: Arc[Mutex[State]] = Arc.create(Mutex.create(initial_state))
```

### Comparison with Other Languages

| Feature | tomi.u | Rust | Go | Swift |
|---------|--------|------|-----|-------|
| **Concurrency** | Async/Await + Actors + Structured | Async/Await | Goroutines + Channels | Structured Async |
| **Memory** | Borrow Checker | Borrow Checker | Optimized GC | ARC |
| **Null Safety** | Option[T] | Option<T> | nil (unsafe) | Optionals |
| **Type Inference** | Full | Full | Partial | Full |
| **Philosophy** | Safety + Expressiveness | Total Control | Simplicity | Safety & Ease |

### Why No Garbage Collector?

tomi.u follows Rust's approach rather than Go/Java's GC model:

| Approach | Pros | Cons |
|----------|------|------|
| **Garbage Collection** (Go/Java/Kotlin) | Simple mental model; modern GCs are "low-pause" with millisecond interruptions | Unpredictable latency; memory overhead; not suitable for real-time systems |
| **ARC** (Swift/Objective-C) | Predictable; memory freed immediately when unused | Reference cycles require weak references; runtime overhead for counting |
| **Borrow Checker** (tomi.u/Rust) | Zero runtime overhead; deterministic; no pauses | Steeper learning curve; some patterns require restructuring |

tomi.u chose the Borrow Checker because:
- **Predictable Performance**: No GC pauses, ever
- **Lower Memory Footprint**: No GC metadata overhead
- **Compile-Time Guarantees**: Bugs caught before runtime
- **Suitable for All Domains**: From embedded systems to high-performance servers

---

## Design Principles

- **Readability First**: Python-inspired indentation-based syntax with no curly braces or semicolons
- **Safety by Default**: Strict static typing with compile-time null safety
- **Zero-Cost Abstractions**: Memory management without garbage collection
- **Modern Ergonomics**: Type inference, pattern matching, and async/await built-in
- **Data-Native**: Integrated query language for collections and graph structures
- **No Reserved Method Names**: All special behaviors are declared via decorators (`@entrypoint`, `@constructor`, `@destructor`), keeping method naming fully flexible

---

## Basic Syntax

### Program Structure

```tomi.u
# Single-line comment

###
Multi-line comment
spanning multiple lines
###

module MyApp:
    
    import std.io
    import std.collections
    
    @entrypoint
    def main() -> Result[Unit, Error]:
        let message = "Hello, tomi.u!"
        io.println(message)
        Ok(())
```

### Variables and Constants

```tomi.u
# Immutable binding (default) - type inferred
let name = "Alice"
let age = 30

# Explicit type annotation
let count: Int32 = 100

# Mutable binding
mut counter = 0
counter = counter + 1

# Constants (compile-time evaluated)
const MAX_SIZE: Int32 = 1024
const PI: Float64 = 3.14159265359
```

### Functions

```tomi.u
# Basic function with explicit types
def add(x: Int32, y: Int32) -> Int32:
    return x + y

# Type inference for return
def greet(name: String) -> String:
    "Hello, {name}!"  # Last expression is implicit return

# Generic function
def identity[T](value: T) -> T:
    value

# Function with default parameters
def connect(host: String, port: Int32 = 8080, timeout: Duration = 30.seconds) -> Connection:
    Connection.new(host, port, timeout)

# Higher-order function
def map[T, U](items: List[T], transform: def(T) -> U) -> List[U]:
    mut result: List[U] = []
    for item in items:
        result.push(transform(item))
    result

# Lambda expressions
let double = |x: Int32| -> Int32: x * 2
let squares = numbers.map(|n| n * n)  # Type inferred from context
```

### Control Flow

```tomi.u
# If expressions (always return a value)
let status = if temperature > 100:
    "Hot"
elif temperature > 50:
    "Warm"
else:
    "Cold"

# While loops
mut i = 0
while i < 10:
    io.println(i)
    i += 1

# For loops with iterators
for item in collection:
    process(item)

# For with index
for i, item in collection.enumerate():
    io.println("{i}: {item}")

# Range-based for
for i in 0..10:      # 0 to 9
    io.println(i)

for i in 0..=10:     # 0 to 10 (inclusive)
    io.println(i)

# Loop with break and continue
for item in items:
    if item.is_empty():
        continue
    if item == "stop":
        break
    process(item)
```

---

## Type System

### Primitive Types

| Type | Description | Size |
|------|-------------|------|
| `Bool` | Boolean value | 1 byte |
| `Int8`, `Int16`, `Int32`, `Int64` | Signed integers | 1-8 bytes |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned integers | 1-8 bytes |
| `Float32`, `Float64` | Floating-point numbers | 4-8 bytes |
| `Char` | Unicode scalar value | 4 bytes |
| `String` | UTF-8 encoded string | varies |
| `Unit` | Empty type (like void) | 0 bytes |
| `Never` | Uninhabited type | 0 bytes |

### Type Aliases

```tomi.u
type UserId = Int64
type Email = String
type Handler = fn(Request) -> Response
type StringMap[V] = Map[String, V]
```

### Structs

tomi.u uses **decorators** instead of reserved method names for special behaviors:

| Decorator | Purpose |
|-----------|--------|
| `@constructor` | Marks a method as a constructor |
| `@destructor` | Marks a method for cleanup when value is dropped |
| `@entrypoint` | Marks a function as program entry point |

```tomi.u
struct Person:
    name: String
    age: Int32
    email: Option[Email]
    handle: Option[ResourceHandle]

    # Constructor - use any method name you prefer
    @constructor
    def create(name: String, age: Int32) -> Self:
        Self:
            name: name
            age: age
            email: None
            handle: None

    # Alternative constructor
    @constructor
    def with_email(name: String, age: Int32, email: Email) -> Self:
        Self:
            name: name
            age: age
            email: Some(email)
            handle: None

    # Destructor - called when value goes out of scope
    @destructor
    def cleanup(self) -> Unit:
        if let Some(h) = self.handle:
            h.release()

    # Methods
    def greet(self) -> String:
        "Hello, I'm {self.name}"

    # Mutable method
    def birthday(mut self) -> Unit:
        self.age += 1

    # Static method
    def anonymous() -> Self:
        Self.create("Anonymous", 0)

# Instantiation
let alice = Person:
    name: "Alice"
    age: 30
    email: Some("alice@example.com")
    handle: None

# Using constructor
let bob = Person.create("Bob", 25)
let carol = Person.with_email("Carol", 28, "carol@example.com")
```

### Option Type (Null Safety)

```tomi.u
# Option[T] represents a value that may or may not exist
type Option[T] = Some(T) | None

# Usage
def find_user(id: UserId) -> Option[User]:
    if database.has(id):
        Some(database.get(id))
    else:
        None

# Working with options
let user = find_user(42)

# Pattern matching
match user:
    Some(u) => io.println("Found: {u.name}")
    None => io.println("User not found")

# Chainable operations
let name = find_user(42)
    .map(|u| u.name)
    .unwrap_or("Unknown")

# Safe navigation with ?. operator
let email = user?.profile?.email
```

### Sum Types (Discriminated Unions)

```tomi.u
# Basic enum
enum Color:
    Red
    Green
    Blue

# Enum with associated values
enum Result[T, E]:
    Ok(T)
    Err(E)

# Complex sum type
enum Message:
    Text(content: String)
    Image(url: String, width: Int32, height: Int32)
    Video(url: String, duration: Duration)
    Location(lat: Float64, lon: Float64)

# Using sum types
def handle_message(msg: Message) -> String:
    match msg:
        Text(content) => "Text: {content}"
        Image(url, w, h) => "Image {w}x{h}: {url}"
        Video(url, duration) => "Video ({duration}): {url}"
        Location(lat, lon) => "Location: ({lat}, {lon})"

# Recursive sum types
enum Json:
    Null
    Bool(Bool)
    Number(Float64)
    String(String)
    Array(List[Json])
    Object(Map[String, Json])
```

### Generic Types and Constraints

```tomi.u
# Generic struct
struct Stack[T]:
    items: List[T]

    def push(mut self, item: T) -> Unit:
        self.items.push(item)

    def pop(mut self) -> Option[T]:
        self.items.pop()

# Generic with constraints
def print_all[T: Display](items: List[T]) -> Unit:
    for item in items:
        io.println(item.display())

# Multiple constraints
def compare_and_print[T: Ord + Display](a: T, b: T) -> Unit:
    let result = if a > b: "greater" else: "not greater"
    io.println("{a.display()} is {result} than {b.display()}")

# Where clauses for complex constraints
def merge[K, V](map1: Map[K, V], map2: Map[K, V]) -> Map[K, V]
    where K: Hash + Eq,
          V: Clone:
    mut result = map1.clone()
    for key, value in map2:
        result.insert(key, value.clone())
    result
```

### Traits (Interfaces)

```tomi.u
trait Display:
    def display(self) -> String

trait Serializable:
    def serialize(self) -> Bytes
    def deserialize(data: Bytes) -> Result[Self, SerializeError]

trait Iterator[T]:
    def next(mut self) -> Option[T]
    
    # Default implementation
    def count(mut self) -> Int64:
        mut count = 0
        while self.next().is_some():
            count += 1
        count

# Implementing traits
impl Display for Person:
    def display(self) -> String:
        "Person({self.name}, age: {self.age})"

# Generic trait implementation
impl[T: Display] Display for List[T]:
    def display(self) -> String:
        let items = self.map(|x| x.display()).join(", ")
        "[{items}]"
```

### Static Type Inference

```tomi.u
# The compiler infers types from context
let x = 42                        # Int32 (default integer type)
let y = 3.14                      # Float64 (default float type)
let name = "Alice"                # String
let items = [1, 2, 3]             # List[Int32]
let lookup = {"a": 1, "b": 2}     # Map[String, Int32]

# Inference through function calls
def create_pair[T](a: T, b: T) -> (T, T):
    (a, b)

let pair = create_pair(10, 20)    # (Int32, Int32) inferred

# Inference in closures
let numbers: List[Int32] = [1, 2, 3, 4, 5]
let doubled = numbers.map(|n| n * 2)  # n is inferred as Int32

# Type inference with constraints
let result = items
    .filter(|x| x > 2)
    .map(|x| x.to_string())
    .collect()  # List[String] inferred from chain
```

---

## Ownership and Memory Management

tomi.u uses an ownership system similar to Rust, ensuring memory safety without garbage collection.

### Ownership Rules

1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped
3. Values can be borrowed (referenced) without transferring ownership

### Move Semantics

```tomi.u
let s1 = String.from("hello")
let s2 = s1              # s1 is moved to s2, s1 is no longer valid
# io.println(s1)         # Compile error: use of moved value

# Copy types (primitives) are copied, not moved
let x = 5
let y = x                # x is copied to y, both valid
io.println(x)            # OK
```

### Borrowing

```tomi.u
# Immutable borrow (&)
def print_length(s: &String) -> Unit:
    io.println("Length: {s.len()}")

let message = "Hello, World!"
print_length(&message)   # Borrow message
io.println(message)      # message still valid

# Mutable borrow (&mut)
def append_exclaim(s: &mut String) -> Unit:
    s.push('!')

mut greeting = "Hello"
append_exclaim(&mut greeting)
io.println(greeting)     # "Hello!"

# Borrow rules enforced at compile time:
# - Multiple immutable borrows allowed
# - Only one mutable borrow at a time
# - Cannot have mutable and immutable borrows simultaneously
```

### Lifetimes

```tomi.u
# Explicit lifetime annotations
def longest['a](x: &'a String, y: &'a String) -> &'a String:
    if x.len() > y.len():
        x
    else:
        y

# Struct with lifetime
struct Parser['a]:
    input: &'a String
    position: Int32

# Lifetime elision (compiler infers lifetimes in common patterns)
def first_word(s: &String) -> &String:  # Lifetimes inferred
    let end = s.find(' ').unwrap_or(s.len())
    s.slice(0, end)
```

### Smart Pointers

```tomi.u
# Box[T] - heap allocation with single ownership
let boxed: Box[Int32] = Box.new(42)

# Rc[T] - reference counted pointer (single-threaded)
let shared: Rc[String] = Rc.new("shared data")
let clone = shared.clone()  # Increment reference count

# Arc[T] - atomic reference counted (thread-safe)
let atomic: Arc[Mutex[List[Int32]]] = Arc.new(Mutex.new([]))
```

---

## Pattern Matching

### Match Expressions

```tomi.u
# Basic pattern matching
match value:
    0 => "zero"
    1 => "one"
    n if n < 0 => "negative"
    n => "other: {n}"

# Matching on enums
match result:
    Ok(value) => io.println("Success: {value}")
    Err(error) => io.println("Error: {error}")

# Destructuring structs
match person:
    Person(name: "Alice", age, ..) => "Found Alice, age {age}"
    Person(name, age, ..) if age >= 18 => "Adult: {name}"
    Person(name, ..) => "Minor: {name}"

# Matching tuples
let point = (3, 4)
match point:
    (0, 0) => "origin"
    (x, 0) => "on x-axis at {x}"
    (0, y) => "on y-axis at {y}"
    (x, y) => "point at ({x}, {y})"

# Matching lists
match items:
    [] => "empty"
    [single] => "one item: {single}"
    [first, second] => "two items"
    [first, ..rest] => "first: {first}, rest has {rest.len()} items"

# Or patterns
match char:
    'a' | 'e' | 'i' | 'o' | 'u' => "vowel"
    'A'..'Z' => "uppercase"
    _ => "other"
```

### Let Patterns

```tomi.u
# Destructuring in let bindings
let (x, y) = get_coordinates()
let Person(name, age, ..) = get_person()

# If-let for conditional destructuring
if let Some(user) = find_user(id):
    io.println("Found user: {user.name}")

# While-let
while let Some(item) = iterator.next():
    process(item)

# Let-else (must diverge in else branch)
let Some(config) = load_config() else:
    panic("Failed to load configuration")
```

---

## Asynchronous Programming

### Async Functions

```tomi.u
# Async function declaration
async def fetch_data(url: String) -> Result[String, HttpError]:
    let response = await http.get(url)
    let body = await response.text()
    Ok(body)

# Calling async functions
@entrypoint
async def main() -> Result[Unit, Error]:
    let data = await fetch_data("https://api.example.com/data")
    io.println(data?)
    Ok(())
```

### Futures and Combinators

```tomi.u
# Future type represents an async computation
type Future[T] = impl Async[Output = T]

# Combining futures
async def fetch_all(urls: List[String]) -> List[String]:
    # Run all fetches concurrently
    let futures = urls.map(|url| fetch_data(url))
    await Future.join_all(futures)

# Select first completed
async def fetch_with_timeout(url: String, timeout: Duration) -> Result[String, Error]:
    match await Future.select(fetch_data(url), Timer.sleep(timeout)):
        First(result) => result.map_err(|e| Error.from(e))
        Second(_) => Err(Error.Timeout)

# Async iteration
async def process_stream(stream: AsyncStream[Message]) -> Unit:
    for await message in stream:
        handle(message)
```

### Structured Concurrency

```tomi.u
# Task spawning within a scope
async def parallel_work() -> Result[Unit, Error]:
    # All tasks must complete before scope exits
    scope |s|:
        s.spawn(async || task_a())
        s.spawn(async || task_b())
        s.spawn(async || task_c())
    
    io.println("All tasks completed")
    Ok(())

# Channels for communication
async def producer_consumer() -> Unit:
    let (tx, rx) = channel[Int32](buffer_size: 10)
    
    scope |s|:
        s.spawn(async ||:
            for i in 0..100:
                await tx.send(i)
        )
        
        s.spawn(async ||:
            for await value in rx:
                io.println("Received: {value}")
        )
```

### Synchronization Primitives

```tomi.u
# Mutex for exclusive access
let counter: Arc[Mutex[Int32]] = Arc.new(Mutex.new(0))

async def increment(counter: Arc[Mutex[Int32]]) -> Unit:
    let guard = await counter.lock()
    guard.value += 1

# RwLock for read-write access
let data: RwLock[Map[String, Int32]] = RwLock.new({})

async def read_data(data: &RwLock[Map[String, Int32]], key: &String) -> Option[Int32]:
    let guard = await data.read()
    guard.get(key).cloned()

# Semaphore for limiting concurrency
let semaphore = Semaphore.new(permits: 10)

async def limited_operation() -> Unit:
    let permit = await semaphore.acquire()
    # ... perform operation ...
    # permit automatically released when dropped
```

---

## Actor Model

tomi.u natively supports the **Actor Model** for concurrent and distributed computing. Actors are isolated units of computation that communicate exclusively through asynchronous message passing, eliminating shared state and the need for locks.

### Defining Actors

```tomi.u
# Define an actor with its state and message handlers
actor Counter:
    # Private state (isolated, never shared)
    state:
        count: Int64 = 0
        name: String
    
    # Constructor (no reserved names - use @constructor)
    @constructor
    def create(name: String) -> Self:
        Self:
            count: 0
            name: name
    
    # Message handlers
    on Increment:
        self.count += 1
    
    on Decrement:
        self.count -= 1
    
    on GetCount -> Int64:
        self.count
    
    on Reset(value: Int64):
        self.count = value
```

### Message Types

```tomi.u
# Messages are defined as types
message Increment
message Decrement
message Reset(value: Int64)
message GetCount -> Int64    # Message with response

# Complex messages
message ProcessOrder:
    order_id: OrderId
    items: List[Item]
    priority: Priority

message OrderProcessed -> Result[Receipt, OrderError]:
    order_id: OrderId
```

### Spawning and Communicating with Actors

```tomi.u
@entrypoint
async def main() -> Result[Unit, Error]:
    # Spawn an actor (returns an ActorRef)
    let counter = spawn Counter.create("main-counter")
    
    # Send messages (fire-and-forget)
    counter.send(Increment)
    counter.send(Increment)
    counter.send(Increment)
    
    # Send message and await response
    let count = await counter.ask(GetCount)
    io.println("Count: {count}")  # Count: 3
    
    # Send with timeout
    let result = await counter.ask(GetCount, timeout: 5.seconds)
    
    Ok(())
```

### Actor Hierarchies and Supervision

```tomi.u
# Actors can supervise child actors
actor Supervisor:
    state:
        workers: List[ActorRef[Worker]]
    
    # Supervision strategy
    supervision:
        strategy: OneForOne          # or AllForOne, RestForOne
        max_restarts: 3
        within: 1.minute
    
    on StartWorkers(count: Int32):
        for i in 0..count:
            let worker = spawn Worker.create(i) supervised by self
            self.workers.push(worker)
    
    # Handle child failures
    on ChildFailed(child: ActorRef[Any], error: Error) -> SupervisorAction:
        match error:
            RecoverableError(_) => Restart
            FatalError(_) => Stop
            _ => Escalate

actor Worker:
    state:
        id: Int32
    
    @constructor
    def create(id: Int32) -> Self:
        Self { id }
    
    on DoWork(task: Task) -> Result[Output, WorkError]:
        # If this fails, supervisor decides what to do
        self.process(task)
```

### Actor Selection and Discovery

```tomi.u
# Find actors by path
let actor = await system.select("/user/services/database")

# Find actors by pattern
let workers = await system.select_all("/user/workers/*")

# Broadcast to multiple actors
for worker in workers:
    worker.send(Shutdown)

# Actor registry
let registry = system.registry()
registry.register("payment-service", payment_actor)
let payment = await registry.lookup("payment-service")
```

### Distributed Actors

```tomi.u
# Configure actor system for distribution
let system = ActorSystem.new()
    .with_cluster(
        name: "my-cluster",
        seeds: ["node1:2551", "node2:2551"],
        roles: ["worker"]
    )
    .start()

# Remote actor references work transparently
let remote_actor = await system.select("akka://cluster@node2:2551/user/service")
remote_actor.send(ProcessRequest(data))

# Cluster-aware routing
actor LoadBalancer:
    state:
        router: Router[Worker]
    
    @constructor
    def create() -> Self:
        Self:
            router: Router.round_robin()
                .with_cluster_awareness(role: "worker")
    
    on Request(data: Data) -> Response:
        await self.router.route(ProcessData(data))
```

### Stateful Actor Persistence

```tomi.u
# Persistent actors survive restarts
actor PersistentCounter:
    state:
        count: Int64 = 0
    
    persistence:
        id: "counter-{self.id}"
        snapshot_every: 100.events
    
    # Events are persisted before state changes
    on Increment:
        persist CountIncremented:
            self.count += 1
    
    on Decrement:
        persist CountDecremented:
            self.count -= 1
    
    # Recover state from persisted events
    recover CountIncremented:
        self.count += 1
    
    recover CountDecremented:
        self.count -= 1
```

---

## Aspect-Oriented Programming

tomi.u natively supports **Aspect-Oriented Programming (AOP)** for separating cross-cutting concerns from business logic. Aspects allow you to inject behavior before, after, or around function execution without modifying the original code.

### Defining Aspects

```tomi.u
# Define an aspect
aspect Logging:
    # Pointcut: defines where the aspect applies
    pointcut logged_functions = execution(def *.*(..)) && @annotated(Log)
    
    # Advice: defines what to do
    before logged_functions(ctx: JoinPoint):
        io.println("[LOG] Entering: {ctx.function_name}")
        io.println("[LOG] Arguments: {ctx.args}")
    
    after logged_functions(ctx: JoinPoint, result: Any):
        io.println("[LOG] Exiting: {ctx.function_name}")
        io.println("[LOG] Result: {result}")
    
    # Around advice for full control
    around logged_functions(ctx: JoinPoint) -> Any:
        let start = Time.now()
        io.println("[LOG] Starting: {ctx.function_name}")
        
        let result = ctx.proceed()  # Execute original function
        
        let duration = Time.now() - start
        io.println("[LOG] Completed in {duration}")
        result
```

### Applying Aspects with Annotations

```tomi.u
# Use annotations to mark functions for aspects
@Log
def process_order(order: Order) -> Result[Receipt, Error]:
    # Business logic here
    validate(order)?
    charge_payment(order)?
    create_receipt(order)

@Log
@Retry(max_attempts: 3)
async def fetch_data(url: String) -> Result[Data, HttpError]:
    await http.get(url).json()
```

### Built-in Aspect Annotations

```tomi.u
# Timing aspect
@Timed
def expensive_calculation() -> Int64:
    # Automatically measures and reports execution time
    compute_result()

# Caching aspect
@Cached(ttl: 5.minutes)
def get_user(id: UserId) -> Option[User]:
    database.find_user(id)

# Retry aspect
@Retry(max_attempts: 3, backoff: Exponential(base: 100.ms))
async def unreliable_service_call() -> Result[Response, Error]:
    await external_api.call()

# Circuit breaker
@CircuitBreaker(failure_threshold: 5, reset_timeout: 30.seconds)
async def protected_call() -> Result[Data, Error]:
    await remote_service.fetch()

# Rate limiting
@RateLimit(requests: 100, per: 1.minute)
async def api_endpoint(request: Request) -> Response:
    handle_request(request)
```

### Pointcut Expressions

```tomi.u
aspect SecurityAspect:
    # Match by function name pattern
    pointcut admin_functions = execution(def admin_*(..))
    
    # Match by module
    pointcut service_calls = execution(def services.*.*(..)) 
    
    # Match by annotation
    pointcut secured = @annotated(RequireAuth)
    
    # Match by argument types
    pointcut user_operations = execution(def *(user: User, ..))
    
    # Match by return type
    pointcut result_functions = execution(def *(..) -> Result[*, *])
    
    # Combine pointcuts
    pointcut critical = admin_functions && secured
    
    # Exclude patterns
    pointcut monitored = service_calls && !execution(def *_internal(..))
    
    before critical(ctx: JoinPoint):
        let user = ctx.get_context[AuthContext]().user
        if !user.has_role(Role.Admin):
            panic("Unauthorized access to admin function")
```

### Aspect for Transaction Management

```tomi.u
aspect Transactional:
    pointcut transactional_methods = @annotated(Transaction)
    
    around transactional_methods(ctx: JoinPoint) -> Any:
        let tx = database.begin_transaction()
        
        match ctx.proceed():
            Ok(result) =>
                tx.commit()
                Ok(result)
            Err(error) =>
                tx.rollback()
                Err(error)

# Usage
@Transaction
def transfer_funds(from: Account, to: Account, amount: Decimal) -> Result[Unit, BankError]:
    from.withdraw(amount)?
    to.deposit(amount)?
    Ok(())
```

### Custom Aspect Annotations

```tomi.u
# Define custom annotation
annotation Audit:
    level: AuditLevel = AuditLevel.Info
    include_args: Bool = true

# Aspect that uses custom annotation
aspect AuditAspect:
    pointcut audited = @annotated(Audit)
    
    after audited(ctx: JoinPoint, result: Any):
        let annotation = ctx.get_annotation[Audit]()
        let entry = AuditEntry:
            function: ctx.function_name
            user: ctx.get_context[UserContext]()?.user_id
            timestamp: Time.now()
            args: if annotation.include_args: Some(ctx.args) else: None
            result: result.to_string()
        
        audit_log.write(entry, level: annotation.level)

# Usage
@Audit(level: AuditLevel.Critical, include_args: true)
def delete_user(user_id: UserId) -> Result[Unit, Error]:
    database.delete(user_id)
```

### Compile-Time Aspect Weaving

```tomi.u
# Aspects are woven at compile time for zero runtime overhead
# The compiler transforms:

@Log
def original(x: Int32) -> Int32:
    x * 2

# Into (conceptually):

def original(x: Int32) -> Int32:
    io.println("[LOG] Entering: original")
    io.println("[LOG] Arguments: (x: {x})")
    let __result = x * 2
    io.println("[LOG] Exiting: original")
    io.println("[LOG] Result: {__result}")
    __result
```

### Aspect Ordering and Priority

```tomi.u
# Control the order of aspect application
@priority(1)  # Lower number = higher priority (runs first)
aspect Authentication:
    # Runs before other aspects
    before secured_endpoints(ctx: JoinPoint):
        verify_token(ctx)

@priority(2)
aspect Authorization:
    # Runs after Authentication
    before secured_endpoints(ctx: JoinPoint):
        check_permissions(ctx)

@priority(10)
aspect Logging:
    # Runs last
    around all_functions(ctx: JoinPoint) -> Any:
        log_entry(ctx)
        let result = ctx.proceed()
        log_exit(ctx, result)
        result
```

---

## Reflection and Runtime Modification

tomi.u provides powerful **reflection** capabilities that integrate seamlessly with the aspect-oriented features, allowing runtime introspection and dynamic behavior modification of types.

### Type Introspection

```tomi.u
import std.reflect

def inspect_type[T]() -> Unit:
    let type_info = reflect.type_of[T]()
    
    io.println("Type: {type_info.name}")
    io.println("Module: {type_info.module}")
    io.println("Size: {type_info.size_bytes} bytes")
    
    # Inspect fields
    for field in type_info.fields():
        io.println("  Field: {field.name}: {field.type_name}")
        for attr in field.attributes():
            io.println("    @{attr.name}")
    
    # Inspect methods
    for method in type_info.methods():
        let decorators = method.decorators().map(|d| "@{d.name}").join(", ")
        io.println("  Method: {method.name}({method.signature}) [{decorators}]")
```

### Dynamic Method Invocation

```tomi.u
def call_dynamic(obj: Any, method_name: String, args: List[Any]) -> Result[Any, ReflectError]:
    let type_info = reflect.type_of_value(obj)
    let method = type_info.get_method(method_name)?
    
    method.invoke(obj, args)

# Usage
let person = Person.create("Alice", 30)
let greeting = call_dynamic(person, "greet", [])?  # "Hello, I'm Alice"
```

### Runtime Behavior Modification via Aspects

Reflection enables dynamic application of aspects at runtime, extending the compile-time AOP capabilities:

```tomi.u
import std.reflect
import std.aspects

# Dynamically apply aspects to existing types
def add_logging_to_type[T]() -> Unit:
    let type_info = reflect.type_of[T]()
    
    for method in type_info.methods():
        # Apply logging aspect to all public methods
        if method.is_public():
            aspects.apply_around(method, |ctx|:
                io.println("[DYNAMIC LOG] Calling {ctx.function_name}")
                let result = ctx.proceed()
                io.println("[DYNAMIC LOG] Completed {ctx.function_name}")
                result
            )

# Runtime aspect registration
def setup_monitoring() -> Unit:
    let aspect = aspects.create_dynamic(
        name: "PerformanceMonitor",
        pointcut: "execution(def services.*.*(..))",
        around: |ctx|:
            let start = Time.now()
            let result = ctx.proceed()
            metrics.record(ctx.function_name, Time.now() - start)
            result
    )
    
    aspects.register(aspect)
```

### Modifying Struct Behavior

```tomi.u
# Add methods to existing types at runtime
def extend_type() -> Unit:
    let type_info = reflect.type_of[Person]()
    
    # Add a new method dynamically
    type_info.add_method(
        name: "full_description",
        implementation: |self: &Person| -> String:
            "{self.name}, age {self.age}, email: {self.email.unwrap_or("N/A")}"
    )
    
    # Add decorator to existing method
    let greet_method = type_info.get_method("greet").unwrap()
    greet_method.add_decorator(Cached(ttl: 1.minute))

# Intercept constructor calls
def intercept_construction[T]() -> Unit:
    let type_info = reflect.type_of[T]()
    
    for ctor in type_info.constructors():  # Methods with @constructor
        aspects.apply_after(ctor, |ctx, result|:
            io.println("Created new {type_info.name}")
            audit_log.record_creation(type_info.name, result)
            result
        )

# Intercept destructor calls
def intercept_destruction[T]() -> Unit:
    let type_info = reflect.type_of[T]()
    
    for dtor in type_info.destructors():  # Methods with @destructor
        aspects.apply_before(dtor, |ctx|:
            io.println("Destroying {type_info.name}")
            audit_log.record_destruction(type_info.name, ctx.self_ref)
        )
```

### Attribute-Based Reflection

```tomi.u
# Define custom attributes
attribute Validate:
    rules: List[ValidationRule]

attribute Serialize:
    format: SerializeFormat = SerializeFormat.Json
    rename: Option[String] = None

# Use reflection to process attributes
def auto_validate[T](obj: T) -> Result[Unit, ValidationError]:
    let type_info = reflect.type_of[T]()
    
    for field in type_info.fields():
        if let Some(validate) = field.get_attribute[Validate]():
            let value = field.get_value(obj)
            for rule in validate.rules:
                rule.check(value)?
    
    Ok(())

# Auto-generate serialization based on attributes
def to_json[T: Reflect](obj: T) -> Json:
    let type_info = reflect.type_of[T]()
    mut result: Map[String, Json] = {}
    
    for field in type_info.fields():
        let serialize = field.get_attribute[Serialize]()
            .unwrap_or(Serialize.default())
        
        let key = serialize.rename.unwrap_or(field.name)
        let value = field.get_value(obj)
        result.insert(key, value.to_json())
    
    Json.Object(result)
```

### Proxy Generation

```tomi.u
# Create dynamic proxies for interfaces
def create_proxy[T: trait](
    handler: def(method: MethodInfo, args: List[Any]) -> Any
) -> T:
    reflect.create_proxy[T](handler)

# Usage: Create a logging proxy
let logged_service: UserService = create_proxy[UserService](|method, args|:
    io.println("Calling {method.name} with {args}")
    let result = method.invoke_default(args)
    io.println("Result: {result}")
    result
)

# Usage: Create a mock for testing
let mock_repo: UserRepository = create_proxy[UserRepository](|method, args|:
    match method.name:
        "find" => Some(User.create("test", 0))
        "save" => Ok(())
        _ => panic("Unexpected call: {method.name}")
)
```

---

## Integrated Query Language (TQL)

tomi.u includes **TQL** (tomi.u Query Language), an integrated query language inspired by SQL and Cypher, designed for querying collections and traversing graphs.

### Collection Queries

```tomi.u
# Basic query syntax
let adults = query:
    from person in people
    where person.age >= 18
    select person

# Projection
let names = query:
    from person in people
    select person.name

# Complex projections
let summary = query:
    from order in orders
    select:
        customer: order.customer.name
        total: order.items.sum(|i| i.price)
        date: order.created_at

# Filtering with multiple conditions
let vip_orders = query:
    from order in orders
    where order.total > 1000
      and order.customer.tier == Tier.Premium
      and order.status != Status.Cancelled
    select order

# Ordering
let sorted = query:
    from product in products
    where product.in_stock
    order by product.price desc, product.name asc
    select product

# Limiting results
let top_ten = query:
    from user in users
    order by user.score desc
    take 10
    select user

# Skip and take (pagination)
let page = query:
    from item in items
    skip page_number * page_size
    take page_size
    select item
```

### Aggregations

```tomi.u
# Group by with aggregation
let sales_by_region = query:
    from sale in sales
    group by sale.region into region_sales
    select:
        region: region_sales.key
        total: region_sales.sum(|s| s.amount)
        count: region_sales.count()
        average: region_sales.avg(|s| s.amount)

# Multiple aggregations
let stats = query:
    from product in products
    aggregate:
        min_price: min(product.price)
        max_price: max(product.price)
        avg_price: avg(product.price)
        total_stock: sum(product.quantity)

# Having clause
let big_categories = query:
    from product in products
    group by product.category into category_products
    having category_products.count() > 10
    select:
        category: category_products.key
        count: category_products.count()
```

### Joins

```tomi.u
# Inner join
let order_details = query:
    from order in orders
    join customer in customers on order.customer_id == customer.id
    select:
        order_id: order.id
        customer_name: customer.name
        total: order.total

# Left join
let all_customers = query:
    from customer in customers
    left join order in orders on customer.id == order.customer_id
    select:
        customer: customer.name
        last_order: order?.created_at

# Multiple joins
let full_report = query:
    from order in orders
    join customer in customers on order.customer_id == customer.id
    join product in products on order.product_id == product.id
    where order.date >= start_date
    select:
        customer: customer.name
        product: product.name
        quantity: order.quantity
        subtotal: order.quantity * product.price
```

### Graph Queries (Cypher-inspired)

```tomi.u
# Define graph types
graph SocialNetwork:
    nodes:
        Person(name: String, age: Int32)
        Company(name: String, industry: String)
        Post(content: String, created_at: DateTime)
    
    edges:
        Follows(Person -> Person, since: DateTime)
        WorksAt(Person -> Company, role: String, since: DateTime)
        Created(Person -> Post)
        Likes(Person -> Post, at: DateTime)

# Basic graph query - find friends
let friends = graph_query:
    match (me:Person)-[Follows]->(friend:Person)
    where me.name == "Alice"
    return friend

# Multi-hop traversal
let friends_of_friends = graph_query:
    match (me:Person)-[Follows]->(friend:Person)-[Follows]->(fof:Person)
    where me.name == "Alice"
      and fof != me
    return distinct fof

# Path queries
let connection_path = graph_query:
    match path = shortest_path((a:Person)-[Follows*1..6]->(b:Person))
    where a.name == "Alice" and b.name == "Bob"
    return path

# Pattern with edge properties
let recent_follows = graph_query:
    match (follower:Person)-[f:Follows]->(followed:Person)
    where f.since >= "2025-01-01"
    return follower.name, followed.name, f.since

# Aggregation in graph queries
let influencers = graph_query:
    match (person:Person)<-[Follows]-(follower:Person)
    return person.name, count(follower) as follower_count
    order by follower_count desc
    limit 10

# Complex patterns
let coworkers_who_interact = graph_query:
    match (p1:Person)-[WorksAt]->(c:Company)<-[WorksAt]-(p2:Person),
          (p1)-[Follows]->(p2)
    where p1 != p2
    return p1.name, p2.name, c.name

# Creating graph data
graph_mutation:
    create (alice:Person {name: "Alice", age: 30})
    create (bob:Person {name: "Bob", age: 28})
    create (alice)-[Follows {since: now()}]->(bob)

# Updating graph data
graph_mutation:
    match (person:Person)-[w:WorksAt]->(company:Company)
    where person.name == "Alice" and company.name == "OldCorp"
    delete w
    create (person)-[WorksAt {role: "Engineer", since: now()}]->(:Company {name: "NewCorp"})
```

### Query Interpolation and Composition

```tomi.u
# Query variables
def find_by_status(status: Status) -> List[Order]:
    query:
        from order in orders
        where order.status == @status  # @ for parameter interpolation
        select order

# Composing queries
let base_query = query:
    from product in products
    where product.active

let cheap_products = query:
    from product in @base_query
    where product.price < 100
    select product

# Async queries
async def search_database(term: String) -> List[Result]:
    await query:
        from item in database.items
        where item.title.contains(@term)
           or item.description.contains(@term)
        order by item.relevance desc
        take 50
        select item
```

---

## Standard Library Overview

### Core Modules

| Module | Description |
|--------|-------------|
| `std.io` | Input/output operations |
| `std.collections` | List, Map, Set, Queue, etc. |
| `std.string` | String manipulation |
| `std.math` | Mathematical functions |
| `std.time` | Date, time, duration |
| `std.fs` | File system operations |
| `std.net` | Networking (TCP, UDP, HTTP) |
| `std.json` | JSON parsing and serialization |
| `std.crypto` | Cryptographic primitives |
| `std.regex` | Regular expressions |
| `std.graph` | Graph data structures |

### Collections

```tomi.u
import std.collections.*

# List
let list: List[Int32] = [1, 2, 3, 4, 5]
list.push(6)
list.map(|x| x * 2)
list.filter(|x| x > 3)
list.fold(0, |acc, x| acc + x)

# Map
let map: Map[String, Int32] = {"a": 1, "b": 2}
map.get("a")           # Option[Int32]
map.insert("c", 3)
map.remove("b")

# Set
let set: Set[Int32] = {1, 2, 3}
set.contains(2)        # true
set.insert(4)
set.union(other_set)

# Queue and Stack
let queue: Queue[String] = Queue.new()
queue.enqueue("first")
queue.dequeue()        # Option[String]

let stack: Stack[Int32] = Stack.new()
stack.push(1)
stack.pop()            # Option[Int32]
```

---

## Python Interoperability

> **Availability:** All tomi.u versions under 1.x.x

tomi.u provides a native bridge to Python 3.14 through the built-in `python` library, enabling seamless interoperability with the Python ecosystem.

### Importing the Python Bridge

```tomi.u
import python
```

### Calling Python Functions

```tomi.u
import python

@entrypoint
def main() -> Result[Unit, Error]:
    # Import a Python module
    let np = python.import("numpy")
    
    # Call Python functions directly
    let array = np.array([1, 2, 3, 4, 5])
    let mean = np.mean(array)
    
    io.println("Mean: {mean}")
    Ok(())
```

### Type Conversions

The `python` library automatically handles type conversions between tomi.u and Python:

| tomi.u Type | Python Type |
|-------------|-------------|
| `Int32`, `Int64` | `int` |
| `Float32`, `Float64` | `float` |
| `Bool` | `bool` |
| `String` | `str` |
| `List[T]` | `list` |
| `Map[K, V]` | `dict` |
| `Option[T]` | `T` or `None` |
| `Bytes` | `bytes` |

### Working with Python Objects

```tomi.u
import python

def data_analysis() -> Result[Unit, Error]:
    let pd = python.import("pandas")
    let plt = python.import("matplotlib.pyplot")
    
    # Create a DataFrame
    let df = pd.DataFrame({
        "name": ["Alice", "Bob", "Charlie"],
        "age": [30, 25, 35],
        "score": [85.5, 92.0, 78.5]
    })
    
    # Use pandas methods
    let filtered = df.query("age > 26")
    let avg_score = df["score"].mean()
    
    # Call matplotlib
    plt.figure(figsize: (10, 6))
    plt.bar(df["name"], df["score"])
    plt.savefig("scores.png")
    
    Ok(())
```

### Async Python Interop

```tomi.u
import python

async def fetch_with_aiohttp(url: String) -> Result[String, Error]:
    let aiohttp = python.import("aiohttp")
    
    # Python async functions are automatically bridged
    let session = await aiohttp.ClientSession()
    let response = await session.get(url)
    let text = await response.text()
    await session.close()
    
    Ok(text.to_tomi_string())
```

### Defining Python-Callable Functions

```tomi.u
import python

# Export a tomi.u function to Python
@python.export
def calculate_fibonacci(n: Int32) -> Int64:
    if n <= 1:
        n as Int64
    else:
        calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)

# Use in Python callbacks
def register_callback() -> Unit:
    let event_system = python.import("my_python_lib.events")
    event_system.on_data(|data: python.PyObject|:
        let value = data.get("value").as_int()
        io.println("Received: {value}")
    )
```

### Python Environment Management

```tomi.u
import python

def setup_environment() -> Result[Unit, Error]:
    # Check Python version (must be 3.14)
    let version = python.version()
    assert(version.starts_with("3.14"), "Requires Python 3.14")
    
    # Install packages at runtime (optional)
    python.pip_install("requests", "numpy", "pandas")
    
    # Set Python path
    python.add_path("/custom/python/modules")
    
    Ok(())
```

### Error Handling with Python

```tomi.u
import python

def safe_python_call() -> Result[Int32, Error]:
    let result = python.try(||:
        let mod = python.import("some_module")
        mod.risky_function()
    )
    
    match result:
        Ok(value) => Ok(value.as_int())
        Err(python.PythonError(e)) => 
            Err(Error.External("Python error: {e.message}"))
```

---

## Error Handling

### Result Type

```tomi.u
# Result type for recoverable errors
type Result[T, E] = Ok(T) | Err(E)

def divide(a: Float64, b: Float64) -> Result[Float64, MathError]:
    if b == 0.0:
        Err(MathError.DivisionByZero)
    else:
        Ok(a / b)

# Using Result
let result = divide(10.0, 2.0)
match result:
    Ok(value) => io.println("Result: {value}")
    Err(error) => io.println("Error: {error}")

# The ? operator for early return
def calculate() -> Result[Float64, MathError]:
    let x = divide(10.0, 2.0)?    # Returns Err early if failed
    let y = divide(x, 3.0)?
    Ok(y)

# Result combinators
let value = divide(10.0, 2.0)
    .map(|x| x * 2)
    .and_then(|x| divide(x, 4.0))
    .unwrap_or(0.0)
```

### Panic and Recovery

```tomi.u
# Unrecoverable errors
def critical_operation() -> Unit:
    if !sanity_check():
        panic("Critical failure: sanity check failed")

# Assert for debugging
assert(x > 0, "x must be positive")
debug_assert(expensive_check())  # Only in debug builds

# Catching panics (boundary operations)
let result = catch_panic(||:
    risky_operation()
)
match result:
    Ok(value) => process(value)
    Err(panic_info) => log_error(panic_info)
```

### Exception Handling

tomi.u supports both Java/JavaScript-style `try/catch` and Python-style `try/except` syntax:

```tomi.u
# Java/JavaScript style with catch
def read_file(path: String) -> Result[String, Error]:
    try:
        let handle = File.open(path)
        let content = handle.read_all()
        return Ok(content)
    catch FileNotFoundError:
        return Err(Error.NotFound(path))
    catch PermissionError as e:
        log.error("Permission denied: {e}")
        return Err(Error.Permission(e))
    finally:
        cleanup_resources()

# Python style with except (equivalent functionality)
def process_data(data: String) -> Unit:
    try:
        let parsed = parse(data)
        process(parsed)
    except ParseError as e:
        log.warn("Parse failed: {e}")
    except:
        log.error("Unexpected error")

# Raise exceptions
def validate(value: Int32) -> Unit:
    if value < 0:
        raise ValueError("Value must be non-negative")
```

### Custom Error Types

```tomi.u
enum AppError:
    NotFound(resource: String)
    PermissionDenied(action: String)
    ValidationFailed(field: String, message: String)
    NetworkError(cause: Box[Error])

impl Display for AppError:
    def display(self) -> String:
        match self:
            NotFound(r) => "Resource not found: {r}"
            PermissionDenied(a) => "Permission denied for: {a}"
            ValidationFailed(f, m) => "Validation failed for {f}: {m}"
            NetworkError(e) => "Network error: {e.display()}"

impl Error for AppError:
    def source(self) -> Option[&Error]:
        match self:
            NetworkError(cause) => Some(cause.as_ref())
            _ => None
```

---

## File Extension

tomi.u source files use the `.tu` extension.

```
src/
  main.tu
  lib.tu
  utils/
    helpers.tu
    validators.tu
```

---

## Example Program

```tomi.u
###
A complete example demonstrating tomi.u features:
A simple REST API for managing a social network
###

module social_api:

import std.net.http.*
import std.json
import std.time.DateTime

# Domain types
struct User:
    id: UserId
    name: String
    email: Email
    created_at: DateTime

enum ApiError:
    NotFound(String)
    InvalidInput(String)
    DatabaseError(String)

type ApiResult[T] = Result[T, ApiError]

# Graph schema
graph Social:
    nodes:
        User(id: UserId, name: String)
    edges:
        Follows(User -> User, since: DateTime)

# Repository trait
trait UserRepository:
    async def find(self, id: UserId) -> ApiResult[User]
    async def create(self, user: User) -> ApiResult[User]
    async def find_followers(self, id: UserId) -> ApiResult[List[User]]

# HTTP handlers
async def get_user(repo: &impl UserRepository, id: UserId) -> Response:
    match await repo.find(id):
        Ok(user) => Response.json(user)
        Err(NotFound(_)) => Response.not_found()
        Err(e) => Response.internal_error(e.display())

async def get_followers(repo: &impl UserRepository, id: UserId) -> Response:
    let followers = await repo.find_followers(id)?
    
    # Using TQL to transform results
    let summary = query:
        from user in followers
        order by user.name
        select:
            id: user.id
            name: user.name
    
    Response.json(summary)

async def get_mutual_followers(graph: &Social, user_a: UserId, user_b: UserId) -> Response:
    let mutual = graph_query:
        match (a:User)<-[Follows]-(mutual:User)-[Follows]->(b:User)
        where a.id == @user_a and b.id == @user_b
        return mutual
    
    Response.json(mutual)

# Main application
@entrypoint
async def main() -> Result[Unit, Error]:
    let config = Config.from_env()?
    let db = Database.connect(config.db_url).await?
    let repo = UserRepositoryImpl.create(db)
    
    let router = Router.new()
        .get("/users/:id", |req| get_user(&repo, req.param("id")))
        .get("/users/:id/followers", |req| get_followers(&repo, req.param("id")))
        .get("/users/:a/mutual/:b", |req|:
            get_mutual_followers(&graph, req.param("a"), req.param("b"))
        )
    
    let server = Server.bind(config.address).await?
    io.println("Listening on {config.address}")
    await server.serve(router)
    
    Ok(())
```

---

## Summary

tomi.u combines the best aspects of modern programming languages:

- **Python's readability** with clean, indentation-based syntax
- **C#'s type safety** with strict static typing and null safety
- **Rust's memory safety** without garbage collection
- **Modern features** like pattern matching, async/await, and type inference
- **Native data capabilities** with integrated SQL/Cypher-inspired query language

This creates a language optimized for building reliable, efficient, and maintainable systems.
