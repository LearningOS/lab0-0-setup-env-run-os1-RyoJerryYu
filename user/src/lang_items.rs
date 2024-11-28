#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "User Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }

    loop {}
}