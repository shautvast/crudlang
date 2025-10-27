# crud-lang

## What is this?
- an experimental language for CRUD applications (backend only though, I think)
- Enterprise as a first-class citizen
  - built in types for dates and uuid for example
  - collection literals
  - ease of use for CRUD operations, like automatic mapping from sql rows to json
- a simple, yet powerful language
- urls are made up of directories and filenames
- a controller sourcefile is a file with the .ctl extension
- likewise:
    - .svc services
    - .cl service clients (that call other services)
    - .dao database access code (not objects)
    - .qc queueconsumers
    - .qp queueproducers
    - .utl utilities
- there is a strict calling hierarchy. A service can not call a controller. It can only go 'down'.
- Services can not call other services, because that is the recipe for spaghetti. Refactor your logic, abstract and put lower level code in utilities.
- Utilities are allowed to call other utilities. OMG, spaghetti after all! TBD
- Automatic memory management using an arena per call

- It is an interpreter written in rust. OMG!
- And it has everything I like in other languages
    - strictly typed
    - [] is a list
    - {} is a map
    - no objects, no inheritance
    - structs and duck typing
    - everything is an expression
    - nice iterators.
    - First class functions? Maybe...
    - automatic mapping from database to object to json
    - indenting like python

**types**

- u32, i32
- u64, i64
- f32, f64,
- string, bool, char
- struct, enum
- date

## open questions
- how to model http headers
- pluggability for middleware?, implement later?
- JWT tokens, I guess

## the example in /src:
- a very simple api that listens to GET /api/customers{:id} and returns a customer from the database

## Design
* heavily inspired by Crafting Interpreters
* language influences from rust and python
* compiler first creates an AST and then compiles to bytecode (no file format yet)
* uses a stack-based virtual machine

## Current status:
* compiler and runtime are still limited but working
* supports:
  * basic types:
    * 32/64 bit integers, signed and unsigned
    * 32/64 bit floats
    * strings
    * bools
    * chars
  * type checking and type inference (although it needs more testing)
  * arithmetic expressions
  * function declaration and calling
  * indenting like python (for now just 1 level, but both tabs or double spaces)
  * strict typing like in rust (no implicit numeric conversions)
  * basic set of operators, including logical and/or and bitwise operations
  
## What's next?
- collection types: list and map
- object/struct types
- control flow
- tests