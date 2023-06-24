#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
