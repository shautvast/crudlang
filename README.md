# crud-lang

_This is now in first-draft phase. Meaning, I just had the idea and I am jotting down very preliminary design decisions._

- an experimental language for CRUD applications (backend only though, I think)
- Enterprise as a first-class citizen
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

**types**

- u32, i32
- u64, i64
- f32, f64,
- string, bool, char
- struct enum
- date

**question**

- how to model headers
- middleware, implement later
- JWT tokens, I guess

**the example im /src: **

- a very simple api that listens to GET /api/customers{:id} and returns a customer from the database
