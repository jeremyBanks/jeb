# Test Specification

This is a simple test specification to verify tracey integration works.

## Greeting Functions

r[test.hello status=stable level=must]
The system MUST provide a function that greets users with a friendly message.

r[test.goodbye status=stable level=should]
The system SHOULD provide a function that says goodbye to users.

## Mathematical Operations

r[test.math.add status=stable level=must]
The system MUST provide a function that adds two numbers together and returns the result.

r[test.math.multiply status=draft level=may]
The system MAY provide a function that multiplies two numbers.

## Repository Testing

r[test.repo.temporary status=stable level=must]
The system MUST provide a way to create temporary git repositories for testing purposes.

r[test.repo.cleanup status=stable level=must]
Temporary repositories MUST be automatically cleaned up when no longer needed.
