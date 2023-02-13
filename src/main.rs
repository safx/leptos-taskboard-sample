use leptos::*;
use taskboard::*;

fn main() {
    mount_to_body(|cx| view! { cx, <Board /> })
}
