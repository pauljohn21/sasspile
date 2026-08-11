## ADDED Requirements

### Requirement: Event type SHALL be a lightweight handle

Event<T> SHALL be an 8-byte NodeId handle into a unified Arena. Event<T> SHALL implement Clone and Copy semantics for zero-cost passing. Event<T> SHALL NOT hold heap-allocated data.

#### Scenario: Event is 8 bytes

- **WHEN** measuring `std::mem::size_of::<Event<T>>()`
- **THEN** the size SHALL be 8 bytes (NodeId = slotmap DefaultKey)

#### Scenario: Event can be cloned

- **WHEN** an Event<T> is cloned
- **THEN** the clone SHALL refer to the same Arena node

### Requirement: Event map combinator

Event<T> SHALL provide a `map<U>(self, f: impl Fn(&T) -> U) -> Event<U>` method that registers a new Event node in the Arena with a Map source. The original Event SHALL NOT be modified. The transform function SHALL be called when an event propagates through the Map node.

#### Scenario: Map transforms payload

- **WHEN** an Event<i32> with value 5 is mapped with `|x| x * 2`
- **AND** the event is flushed
- **THEN** the downstream Behavior SHALL receive 10

#### Scenario: Chained map

- **WHEN** an Event<i32> is mapped with `|x| x + 1` then `|x| x * 10`
- **AND** the event with value 5 is flushed
- **THEN** the downstream Behavior SHALL receive 60

### Requirement: Event filter combinator

Event<T> SHALL provide a `filter(self, pred: impl Fn(&T) -> bool) -> Event<T>` method. Events that fail the predicate SHALL NOT propagate to downstream nodes.

#### Scenario: Filter blocks non-matching events

- **WHEN** an Event<i32> is filtered with `|x| x > 0`
- **AND** events -1, 5, -3, 2 are flushed
- **THEN** the downstream Behavior SHALL only receive 5 and 2

### Requirement: Event merge combinator

Event<T> SHALL provide a `merge(self, other: Event<T>) -> Event<T>` method that combines two event streams into one. Events from both streams SHALL propagate to the merged Event's downstream.

#### Scenario: Merge combines two streams

- **WHEN** two Event<i32> streams are merged
- **AND** stream A fires 1, stream B fires 10
- **AND** events are flushed
- **THEN** the downstream Behavior SHALL receive both 1 and 10

### Requirement: Event sample combinator

Event<T> SHALL provide a `sample<B>(self, behavior: &Behavior<B>) -> Event<B>` method. When the Event fires, the current value of the Behavior SHALL be read and propagated as the new Event's payload.

#### Scenario: Sample reads behavior value on event

- **WHEN** a Behavior<i32> has value 42
- **AND** a sampling Event fires
- **AND** the event is flushed
- **THEN** the downstream Behavior SHALL receive 42

### Requirement: Behavior type SHALL be a lightweight handle

Behavior<T> SHALL be an 8-byte NodeId handle into a unified Arena. Behavior<T> SHALL implement Clone for zero-cost passing. Behavior<T> SHALL NOT hold heap-allocated data.

#### Scenario: Behavior is 8 bytes

- **WHEN** measuring `std::mem::size_of::<Behavior<T>>()`
- **THEN** the size SHALL be 8 bytes

### Requirement: Behavior now reads current value

Behavior<T> SHALL provide a `now(&self) -> T` method that reads the current value from the Arena. The value SHALL reflect all events that have been flushed so far.

#### Scenario: Now returns initial value

- **WHEN** a Behavior is created with accumulate and initial value 42
- **THEN** `now()` SHALL return 42 before any events are flushed

#### Scenario: Now returns updated value after flush

- **WHEN** a Behavior accumulates events with `|s, n| *s += n`
- **AND** events 1, 5, 3 are fired and flushed
- **THEN** `now()` SHALL return 9

### Requirement: Behavior accumulate creates state from events

Behavior<T> SHALL provide an `accumulate<E>(event: Event<E>, initial: T, update: impl Fn(&mut T, &E)) -> Behavior<T>` method. This is the ONLY way to create a stateful Behavior from events. The update function SHALL be called for each event that propagates to this Behavior.

#### Scenario: Accumulate starts with initial value

- **WHEN** a Behavior is accumulated with initial value 0
- **THEN** `now()` SHALL return 0 before any events

#### Scenario: Accumulate applies updates in order

- **WHEN** a Behavior accumulates with `|s, n| *s += n`
- **AND** events 1, 5, 3 are fired and flushed
- **THEN** `now()` SHALL return 9

### Requirement: Behavior map creates derived behavior

Behavior<T> SHALL provide a `map<U>(self, f: impl Fn(&T) -> U) -> Behavior<U>` method. The derived Behavior SHALL automatically update when the upstream Behavior changes. The initial value SHALL be computed immediately from the upstream's current value.

#### Scenario: Map derives value

- **WHEN** a Behavior<i32> with value 3 is mapped with `|c| format!("Count: {c}")`
- **THEN** `now()` SHALL return "Count: 3"

#### Scenario: Map updates on upstream change

- **WHEN** a Behavior<i32> is mapped with `|c| c * 2`
- **AND** the upstream Behavior is updated to 5 and flushed
- **THEN** the mapped Behavior's `now()` SHALL return 10

#### Scenario: Chained map

- **WHEN** a Behavior<i32> is mapped with `|c| c * 2` then `|d| format!("Value={d}")`
- **AND** the upstream is updated to 5 and flushed
- **THEN** the final mapped Behavior's `now()` SHALL return "Value=10"

### Requirement: Runtime SHALL provide unified Arena

Runtime SHALL own a SlotMap Arena containing all Event and Behavior nodes. Runtime SHALL provide `create_event<T>() -> (Event<T>, Emitter<T>)` for creating event sources. Runtime SHALL maintain a dependency graph for event propagation.

#### Scenario: Runtime creates event source

- **WHEN** `rt.create_event::<i32>()` is called
- **THEN** it SHALL return an Event<T> handle and an Emitter<T>

### Requirement: Runtime SHALL batch event processing

Runtime SHALL queue events in a FIFO queue when `Emitter::fire()` is called. Events SHALL NOT propagate immediately. `Runtime::flush()` SHALL drain the queue and propagate all queued events along the dependency graph.

#### Scenario: Events not processed before flush

- **WHEN** an Emitter fires an event
- **AND** `flush()` has not been called
- **THEN** downstream Behaviors SHALL NOT reflect the event

#### Scenario: Multiple events batch processed

- **WHEN** an Emitter fires events 1, 2, 3
- **AND** `flush()` is called
- **THEN** all three events SHALL be processed in order

### Requirement: Emitter SHALL trigger events

Emitter<T> SHALL provide a `fire(&self, payload: T)` method that queues the event in the Runtime. Emitter<T> SHALL implement Clone. Emitter SHALL use a raw pointer to Runtime (SAFETY: Runtime outlives all Emitters within the `runtime()` closure).

#### Scenario: Emitter fire queues event

- **WHEN** `emitter.fire(42)` is called
- **AND** `rt.flush()` is called
- **THEN** the downstream Behavior SHALL receive 42

#### Scenario: Cloned emitter fires same event

- **WHEN** an Emitter is cloned
- **AND** both original and clone fire events
- **AND** `flush()` is called
- **THEN** both events SHALL be received by the downstream

### Requirement: Runtime SHALL provide Context

Runtime SHALL provide `provide_context<T: Clone + 'static>(value: T)` and `use_context<T: Clone + 'static>() -> Option<T>` methods for passing values down the widget tree. Context SHALL be type-indexed by TypeId.

#### Scenario: Provide and use context

- **WHEN** `rt.provide_context(42_i32)` is called
- **THEN** `rt.use_context::<i32>()` SHALL return `Some(42)`

#### Scenario: Use context returns None when not provided

- **WHEN** no value of type i32 has been provided
- **THEN** `rt.use_context::<i32>()` SHALL return `None`

#### Scenario: Context supports multiple types

- **WHEN** values of type i32, String, and Vec<f64> are provided
- **THEN** all three types SHALL be retrievable via `use_context`

### Requirement: runtime function SHALL create Runtime context

The `runtime<F: FnOnce(&'static Runtime) -> R, R>(f: F) -> R` function SHALL create a Runtime and pass a 'static reference to the closure. All Event/Behavior/Emitter operations SHALL occur within this closure.

#### Scenario: runtime closure executes

- **WHEN** `runtime(|rt| { ... })` is called
- **THEN** the closure SHALL receive a valid Runtime reference

#### Scenario: Multiple behaviors from same event

- **WHEN** two Behaviors accumulate from the same Event (via clone)
- **AND** events 3, 7, 2 are fired and flushed
- **THEN** both Behaviors SHALL independently process all events
