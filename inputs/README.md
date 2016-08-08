## lazybox inputs

## Interaction

The process of translating user **inputs** into **actions**.  
**Interaction interfaces** are provided by the application in order to define how it can be interacted with. An interface defines a set of possible actions.
An **interaction profile** is provided by the user in order to define how he wants to access those interfaces. For each interface, it defines a set of **rules** to produce actions.

```yaml
# interaction profile example
spaceship: # interface name
  rules:
    - { action: Shoot, when: MouseButton.Held.Left }
    - { action: LaunchBomb, when: Key.Pressed.Space }
```

Each interface will have an associated action-event that will be triggered according to these rules.
 
## [Documentation](https://lazybox.github.io/lazybox/lazybox_inputs)
