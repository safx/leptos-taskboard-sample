use leptos::{view, mount_to_body};
use taskboard::Board;

fn main() {
    mount_to_body(|| view! { <Board /> })
}
