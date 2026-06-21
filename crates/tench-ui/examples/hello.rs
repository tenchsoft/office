//! Hello world example — shows a window with a Button widget using the native backend.

use tench_ui::platform::native::run_native;
use tench_ui::widgets::Button;

fn main() {
    println!("Starting tench-ui hello example...");

    run_native(|backend| {
        backend.set_root(Button::new("Hello, Tench!").on_click(|| {
            println!("Button clicked!");
        }));
    });
}
