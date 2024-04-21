use std::collections::{HashMap, LinkedList, VecDeque};

const MAX_ITER: i32 = 300000;

fn main() {
    // Vectors
    vec_operations();

    // VecDeque
    vec_deque_operations();

    // LinkedList
    linked_list_operations();

    // HashMap
    hash_map_operations();

    /*
    Expectations (before starting):
    I expect that a Linked List insertion will be slower than vec/vecdeque
    because even though it required O(1) for insert/delete, it is still slower because
    of the following:
    The last element of the list holds a null pointer. So you go through the list
    by jumping from each element to the next. Also, when you want to append a new element,
    you can just allocate memory for that one element anywhere in memory and let the last
    element of the list point to that new element. This means inserting/deleting is O(1).
    You never have to copy and reallocate data when growing your list.
    However, this also means that indexing the list is incredibly slow, because you may have
    to keep jumping through memory, whereas indexing an array always takes the same amount
    of time, because you can just calculate the exact position of the nth element in
    memory (begin + sizeof(int)*n.
    I expect that hashmap will have a similar performance to the first two since it is O(1)
    insert/delete.

    Which collection type was the fastest for adding and removing elements?
    Sequences were by the fastest.

    Why do you think this was the case?
    As mentioned before, inserting and deleting in sequences is fast (at the beginning or at
    the end)because we can calculate the exact position of the nth element in memory.
    The HashMap insert/delete was slower because we need to hash the key in order to get the
    memory location of the value. This requires an additional calculation in the process,
    hence why it was 172ms/131ms compared to 5.37ms/4.24ms for Vecs.

    Is there any significant difference between Vec and VecDeque deletion?
    They are very similar, but theoretically VecDeque should be slower than Vec.
    In practice, VecDeque is marginally slower in insertion, and marginally
    faster in deletion.

    When would you consider using VecDeque over Vec?
    The whole point of VecDeque is to make certain operations faster, namely pushing
    and popping the start of the collection. Vec is very slow at this, especially if
    there are a lot of items, because it involves moving all other items to make space.
    The structure of VecDeque makes these operations fast but at the expense of performance
    of other operations in comparison to Vec.

    When would you consider using LinkedList over Vec?
    I would use a Linked List when implementing a queue or a stack or when implementing graph
    algorithms, since they are better suited for when you need to insert or delete elements
    frequently. Arrays and vectors are better suited for situations where you need to access
    elements at random, as this can be done in constant time.
     */
}

/// measure the insertion and removal
/// operations of a vector
fn vec_operations() {
    let mut vec = Vec::new();

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        vec.push(i);
    }
    let time_end = std::time::Instant::now();

    println!("==== Vector ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = std::time::Instant::now();
    for _ in 0..MAX_ITER {
        vec.remove(0);
    }
    let time_end = std::time::Instant::now();

    println!("remove: {:?}", time_end - time_start);
}

/// measure the insertion and removal
/// operations of a VecDeque
fn vec_deque_operations() {
    let mut vec_deque = VecDeque::new();

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        vec_deque.push_back(i);
    }
    let time_end = std::time::Instant::now();

    println!("==== VecDeque ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = std::time::Instant::now();
    for _ in 0..MAX_ITER {
        vec_deque.pop_front();
    }
    let time_end = std::time::Instant::now();

    println!("remove: {:?}", time_end - time_start);
}
fn linked_list_operations() {
    let mut list = LinkedList::new();

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        list.push_back(i);
    }
    let time_end = std::time::Instant::now();

    println!("==== Linked List ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = std::time::Instant::now();
    for _ in 0..MAX_ITER {
        list.pop_front();
    }
    let time_end = std::time::Instant::now();

    println!("remove: {:?}", time_end - time_start);
}
fn hash_map_operations() {
    let mut map = HashMap::new();

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        map.insert(i, i);
    }
    let time_end = std::time::Instant::now();

    println!("==== Hash Map ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        map.remove(&i);
    }
    let time_end = std::time::Instant::now();

    println!("remove: {:?}", time_end - time_start);
}
