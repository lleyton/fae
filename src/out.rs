use ansi_term::Colour::{Fixed, Red, Yellow};
use std::panic::{set_hook, PanicInfo};

pub fn warning(message: &str) {
    println!("{} {}", Yellow.paint("WARNING"), message);
}

fn panic_message(info: &PanicInfo) {
    if let Some(message) = info.payload().downcast_ref::<String>() {
        println!("{} something caused a panic within fae", Red.paint("ERROR"));
        println!("    {}", Fixed(248).paint(message));
    } else {
        warning("Oops, we couldn't format the panic properly, here is what it says");
        panic!("{:?}", info.payload());
    }
}

/// Registers a new function that is responsible for handling panic messages.
/// This is based off the information provided in the stack overflow post:
///
/// https://stackoverflow.com/questions/51786498/is-there-a-way-to-make-expect-output-a-more-user-friendly-message
pub fn change_panic_message() {
    set_hook(Box::new(panic_message));
}
