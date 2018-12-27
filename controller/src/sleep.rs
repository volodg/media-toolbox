
pub fn sleep() {
    use std::{thread, time};
    let ten_millis = time::Duration::from_millis(1000);
    thread::sleep(ten_millis);
}
