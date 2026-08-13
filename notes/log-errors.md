| Cause | Message | Solution |
|---|---|---|
|using `use` to declare a module| pub use crate::game;^^^^^^^^^^^ no `game` in the root| use `mod`|
|Not implementing `Send` marker trait for type to be move between thread|`(dyn game::player::PlayStrategy + 'static)` cannot be sent between threads safely| add `Send` on the type, `Box<dyn PlayStrategy + Send>`|
