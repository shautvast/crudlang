# crud-lang

## Why?
1. Existing languages are just fine, but building web services is bolted on, instead of supported out-of-the-box.
2. Whereas every company needs an API these days. 
3. Is it just me? -I always have trouble mapping urls the code that handles them.
4. There is no language (AFAIK) that supports layering. (controllers, services, database access, etc). This pattern is ubiquitous (at least where I live). 
5. ORM's are awful. Mapping from sql rows to objects is a pain. This should be easy.
6. Json is ubiquitous. Convention over configuration: A controller returns json by default.
7. Yes, you can automatically serve json from postgres or whatever, but that is not the point. We want to build services.
   
## Now what?
- an experimental language for CRUD applications (web api's)
- Enterprise as a first-class citizen
  - built-in types for dates and uuid for example
  - collection literals
  - ease of use for CRUD operations, like automatic mapping from sql rows to json
- urls are made up of directories. 
- a controller sourcefile is a file named web.crud
- likewise:
    - service.crud for services
    - db.crud database access code 
    - util.crud utilities
- it is not mandatory to have services. If you want, you can put all your logic in a controller.
- and it can only access functions in its own subtree. Generic code should be put higher up in the tree.
- Therefore, services cannot call other services, because that is the recipe for spaghetti. Refactor your logic, abstract and put lower level code in utilities.
- openapi support

### An interpreter written in Rust. 
OMG!
And I cherry picked things I like, mostly from rust and python. 
  - strictly typed
  - [] is a list
  - {} is a map
  - no objects, no inheritance
  - structs and duck typing
  - everything is an expression
  - nice iterators.
  - First-class functions? Maybe...
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
- pluggability for middleware?, implement later?
- JWT tokens, I guess

## the example in /source:
- a very simple api that returns "hello world"
  - but it demonstrates the basic concepts
- it starts an axum server
- go to http://localhost:3000/hello
- goal: it listens to GET /api/customers{:id} and returns a customer from the database

## Design
* heavily inspired by Crafting Interpreters. 
* compiler first creates an AST and then compiles to bytecode (no file format yet)
* uses a stack-based virtual machine

## Current status: infancy
* compiler and runtime are still limited but working
* supports:
  * basic types:
    * 32/64 bit integers, signed and unsigned
    * 32/64 bit floats
    * strings
    * bools
    * chars
    * lists and maps (as literals)
  * type checking and type inference (although it needs more testing)
  * arithmetic expressions
  * function declaration and calling
  * indenting like python (for now just 1 level, but both tabs or double spaces)
  * strict typing like in rust (no implicit numeric conversions)
  * basic set of operators, including logical and/or and bitwise operations
* automatic injection of uri, query parameters and headers
  * if you declare them they will be available in the function body
  * example:
```html
fn get(path: string, headers: map, query: map) -> string:
    "hello" + path
```
* includes a rudimentary REPL
  * ```cargo run -- --repl```)
  * list functions and functions that serve endpoints
  * planned: 
    * edit source files
    * test endpoints
* basic http support (GET, POST, PUT, DELETE)
* watch daemon that recompiles on file changes
  * ```cargo run -- --watch``` 
  
## What's next?
* guards: this will be the way to correctly deal with parameters
```
fn get():
    | path == "/" -> list:
service.get_all()
    | path == "/{uuid}" -> Customer?:
service.get(uuid)?
    | path == "/" && query.firstname -> Customer?:
service.get_by_firstname(fname)?
    | path == "/" && query.last_name -> Customer?
service.get_by_lastname(lname)?
    | 404
```
* this may also require ADT's...
* object/struct types: Work in Progress
* control flow
* test support

## What about performance?
* Clueless really! We'll see.
* But it is written in rust
* And it has no GC
* So, maybe it will compete with python?

## A quick taste
**variables**
```
let a = 42
```
* declares a variable of type i64 (signed 64 bit integer)

or explictly as u32 (unsigned 32 bit integer)
```
let a:u32 = 42
```

* All variables are mutable right now. Have not come to a decision yet about mutable vs immutable variables.
* You must declare a variable before using it. Block scoping.
* There is no ```null```. There is ```void``` though.
* You must initialize a variable when declaring it.

**strings**
```
let b:string = "hello "
```
Strings support concatening with +
```
let c = b + "world"
```

**lists**
```
let list = ["foo", "bar", 1, 1.0]
```
No generic types (yet). A list can hold any type.
* lists support appending with + 
```
let list 2 = list + "baz"
```
_note to self: implement adding 2 lists_

**functions**
```
fn add(a:i64, b:i64) -> i64:
    a + b
```
* Everything is an expression. 
* The result of the last expression is returned.
* There are no semicolons. End-of-line chars serve as delimiters.
* Having multiple expressions on one line is not allowed.
* indenting determines a block and therefore the scope.
* The return type declaration is optional. If not specified, it is void.

**function calling**
```
let sum = add(1,2)
```

**An actual controller**
```
fn get() -> string:
    add("hello", "world")

fn add(a: string, b: string) -> string:
    a + " " + b
```
* get() is the entry point for http GET method calls, likewise for POST, PUT, DELETE, etc.
