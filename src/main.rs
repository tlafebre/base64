mod decode;
mod encode;

fn main() {
    let s = String::from("foob");
    println!("{:?}", encode::encode(s));

    let t = String::from("Zm9vYmE=");
    println!("{:?}", decode::decode(t));
}
