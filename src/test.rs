
fn main() -> () {
    let mut v = vec!(1, 2, 3);
    v.insert(0, 123);
    v.insert(1, 233);
    v.insert(4, 44);
    println!("{:?}", v);
}
