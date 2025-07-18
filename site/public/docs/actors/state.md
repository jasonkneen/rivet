# State

Actor state provides the best of both worlds: it's stored in-memory and persisted automatically. This lets you work with the data without added latency while still being able to survive crashes & upgrades.

**Using External SQL Databases**

Actors can also be used with external SQL databases. This can be useful to integrate actors with existing
applications or for storing relational data. Read more [here](/docs/actors/external-sql).

## Initializing State

There are two ways to define a actor's initial state:

```typescript
// Simple state with a constant
const counter = actor(,
  
  actions: 
});
```

```typescript
// State with initialization logic
const counter = actor(;
  },
  
  actions: 
});
```

The `createState` function is called once when the actor is first created. See [Lifecycle](/docs/actors/lifecycle) for more details.

## Modifying State

To update state, modify the `state` property on the context object (`c.state`) in your actions:

```typescript
const counter = actor(,
  
  actions: ,
    
    add: (c, value) => 
  }
});
```

Only state stored in the `state` object will be persisted. Any other variables or properties outside of this are not persisted.

## State Saves

Actors automatically handle persisting state transparently. This happens at the end of every action if the state has changed.

In the rare occasion you need to force a state change mid-action, you can use `c.saveState()`. This should only be used if your action makes an important state change that needs to be persisted before the action completes.

```typescript
const criticalProcess = actor(,
  
  actions: `);
      
      // Force save state before the async operation
      c.saveState();
      
      // Long-running operation that might fail
      await someRiskyOperation();
      
      // Update state again
      c.state.steps.push(`Completed step $`);
      
      return c.state.currentStep;
    }
  }
});
```

## State Isolation

Each actor's state is completely isolated, meaning it cannot be accessed directly by other actors or clients. This allows actors to maintain a high level of security and data integrity, ensuring that state changes are controlled and predictable.

To interact with a actor's state, you must use [Actions](/docs/actors/actions). Actions provide a controlled way to read from and write to the state.

## Sharing State Between Actors

If you need a shared state between multiple actors, you have two options:

1. Create a actor that holds the shared state that other actors can make action calls to
2. Use an external database, see [External SQL Databases](/docs/actors/external-sql)

## Ephemeral Variables

In addition to persisted state, Rivet provides a way to store ephemeral data that is not saved to permanent storage using `vars`. This is useful for temporary data that only needs to exist while the actor is running or data that cannot be serialized.

`vars` is designed to complement `state`, not replace it. Most actors should use both: `state` for critical business data and `vars` for ephemeral or non-serializable data.

### Initializing Variables

There are two ways to define a actor's initial vars:

```typescript
// Define vars as a constant
const counter = actor(,
  
  // Define ephemeral variables
  vars: ,
  
  actions: 
});
```

When using static `vars`, all values must be compatible with `structuredClone()`. If you need to use non-serializable objects, use `createVars` instead, which allows you to create these objects on the fly.

```typescript
// Define vars with initialization logic
const counter = actor(,
  
  // Define vars using a creation function
  createVars: () => ;
  },
  
  actions: 
});
```

### Using Variables

Vars can be accessed and modified through the context object with `c.vars`:

```typescript
const counter = actor(,
  
  // Create ephemeral objects that won't be serialized
  createVars: () => `);
    });
    
    return ;
  },
  
  actions: 
  }
});
```

### When to Use `vars` vs `state`

In practice, most actors will use both: `state` for critical business data and `vars` for ephemeral, non-serializable, or performance-sensitive data.

Use `vars` when:

- You need to store temporary data that doesn't need to survive restarts
- You need to maintain runtime-only references that can't be serialized (database connections, event emitters, class instances, etc.)

Use `state` when:

- The data must be preserved across actor sleeps, restarts, updates, or crashes
- The information is essential to the actor's core functionality and business logic

## Limitations

State is constrained to the available memory.

Only JSON-serializable types can be stored in state. In serverless runtimes that support it (Rivet, Cloudflare Workers), state is persisted under the hood in a compact, binary format. This is because JavaScript classes cannot be serialized & deserialized.