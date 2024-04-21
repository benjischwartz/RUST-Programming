1) I saw someone's code fail to compile because they 
were trying to send non-thread-safe data across threads. 
How does the Rust language allow for static (i.e. at compile time)
guarantees that specific data can be sent/shared acrosss threads?

Rust allows for these guarantees through its ownership and type system.
It uses the `Send` and `Sync` traits to enforce safe concurrency.
`Send` means a type can be transferred between threads, so they are
thread-safe for sending, while `Sync` can be shared between threads
without data races. 
TLDR: `Send` = transferring ownership across threads
`Sync` = borrowing across threads

2) Do you have to then implement the Send and Sync traits for 
every piece of data (i.e. a struct) you want to share and send across threads?

No, Rust provides automatic implementations of Send and Sync for most types 
that are considered safe to be sent or shared between threads, like primitive
types and standard library types. Custom types will be `Send`/`Sync` if all
their constituent types are also `Send`/`Sync`.
However, there are cases where you might need to explicitly opt out of `Send`
or `Sync` for certain types by implementing these traits manually (e.g., if a
type contains non-thread-safe raw pointers or references that might lead to data
races). In general, Rust's type system and compiler provide strong guarantees 
to ensure safe concurrent programming without requiring excessive manual
implementations of `Send` and `Sync`.

3) What types in the course have I seen that aren't Send? Give one example, 
and explain why that type isn't Send 

`Rc<T>` is not `Send`. This is because it is a reference-counted smart pointer
(like `shared_ptr` in C++). This management of reference counts is non-atomic
If you were to share an `Rc<T>` between multiple threads and try to clone/drop
it from different threads, it could lead to data races.

To safely share reference-counted data, use types like `Arc<T>` which is designed
to be both `Send` and `Sync`.

4) What is the relationship between Send and Sync? Does this relate
to Rust's Ownership system somehow?

5) Are there any types that could be Send but NOT Sync? Is that even possible?

6) Could we implement Send ourselves using safe rust? why/why not?
