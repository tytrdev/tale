# TALE

Ty's
Accessible
Lisp
Environment

## Who?

Me. I'm Ty.

## What?

TALE is a very simple Lisp.
Ideally TALE will mirror clojure syntax wherever possible. This is because clojure is the most popular
Lisp in use today. I want it to be welcoming as well as one day maybe be a full runtime for Clojure.

The environment was implemented using Rust.
TALE will one day support some nifty visualizations about code written using TALE. These visualizations
are the entire point of TALE.

## Why?

I'm experimenting with writing tools to analyze Lisp codebases.
This effort is heavily inspired by the reflective tools of the Pharo Smalltalk environment.
I'm implementing it in Rust for fun.

## When?

Dunno.

## How?

I started by following Stepan Parunashvili's blog about [Risp](https://stopa.io/post/222).
Then I did other things that I haven't done yet. Check below and check back again in the future.

## What's next?

- Cleanup codebase
  - Rework code into modules (done)
  - Use module namespacing to cleanup Types (done)
  - General style refactoring
  - Rework forms to match clojure
- Separate std library implementation 
- Separate REPL
  - Refactor REPL into a module
  - Make continuous input possible through multiple lines
  - Cleanup error messages and make them way more specific
  - Rework entry point to allow options to specify env type
- Add the ability to run files
  - Create a new module
  - Allow the specification of a command line argument to target files
    - Default to files? (Revisit)
- Implement Namespaces
  - Follow clojure namespacing semantics
  - Allow importing of files in the REPL
- Implement Macros
  - Base macro form
  - Macro expansion at runtime
  - Macro expansion for printing?
- Visualization tools
  - Read the code in and generate stats
  - Record stats in some (TBD) useful format
  - Display stats in a useful visual format
  - Allow connection traversal within visualization