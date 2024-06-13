use nanoid::nanoid;
use uuid::Uuid;

fn main() {
    let id = Uuid::new_v4();
    println!("{id}");

    let id = Uuid::now_v7();
    println!("{id}");

    let buf: [u8; 16] = *b"abcdefghijklmnop";
    let id = Uuid::new_v8(buf);
    println!("{id}");

    let id = nanoid!(8);
    println!("{id}");
}
