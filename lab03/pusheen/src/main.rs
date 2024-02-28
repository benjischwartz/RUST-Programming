/*
Write a comment explaining why this program does not compile, and what potential problems Rust is trying to protect you from.
Fix the program so that it compiles.
Write a comment explaining how you enabled the program to compile.
*/

/*
ANSWER:
The program does not compile since we try to borrow 'vec' as a mutable more than once.
We can only ever have one mutable borrow at a time, since if we have multiple mutable
borrows this could result in a data race.
The only case where we can have multiple borrows for the same object is if they are immutable
"shared borrows".
Essentially, concurrency becomes safer: Since Rust guarantees that there's either a single
mutable reference or multiple immutable ones, we won't run into issues of data races.

I changed the program by removing the 'b' borrow, and instead using the same borrow 'a'
to perform both push operations.
 */
fn main() {
    let mut vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let a = &mut vec;

    a.push(11);
    a.push(12);

    for x in a {
        println!("{x}");
    }
}
