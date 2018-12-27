extern crate interactor;
use interactor::*;

fn main() {
    let read_result = read_from_tty(|buf, b, tty| {
        tty.write(&format!("({:?} | {})\n", buf, b).into_bytes());
    }, false, false).unwrap();
    println!("Read: {}", String::from_utf8(read_result).unwrap());

    let chosen_ext = pick_from_list(default_menu_cmd().as_mut(), &["first", "second"], "Selection: ").unwrap();
    println!("Congratulations, you chose '{}'!!", chosen_ext);
}