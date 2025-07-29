dialoguer
Struct Select

Summary
Source
pub struct Select<'a> { /* private fields */ }
Renders a select prompt.

User can select from one or more options. Interaction returns index of an item selected in the order they appear in item invocation or items slice.

Example
use dialoguer::Select;

fn main() {
    let items = vec!["foo", "bar", "baz"];

    let selection = Select::new()
        .with_prompt("What do you choose?")
        .items(&items)
        .interact()
        .unwrap();

    println!("You chose: {}", items[selection]);
}