# crud-lang

## Why?
1. Existing languages are great, but building web services is bolted on, instead of supported out-of-the-box.
2. Whereas every company needs an API these days. 
3. I always have trouble mapping urls the code that handles them.
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
- openapi support

### An interpreter written in Rust. 
OMG!
And it has everything I like in other languages
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
    * lists (as literals)
  * type checking and type inference (although it needs more testing)
  * arithmetic expressions
  * function declaration and calling
  * indenting like python (for now just 1 level, but both tabs or double spaces)
  * strict typing like in rust (no implicit numeric conversions)
  * basic set of operators, including logical and/or and bitwise operations
  
## What's next?
- collection types: --list-- and map
- object/struct types
- control flow
- tests

## A quick taste
**variables**
```
let a = 42
```
* declares a variable of type i64

or explictly as u32
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


**function calling**
```
let sum = add(1,2)
```
