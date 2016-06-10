# external_set

A Rust set data structure with externally-owned items, designed for managing 
subscribers, clients, listeners, or observers across threads.

Inserting an element returns an ItemOwner that maintains ownership of the object.
When the ItemOwner is dropped, the element is removed from the set.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.