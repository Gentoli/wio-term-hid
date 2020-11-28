use arrayvec::Array;
use arrayvec::ArrayString;
use core::fmt::Write;

pub fn str_to_fixed<A: Array<Item = u8> + Copy>(s: &str) -> ArrayString<A> {
    let mut buf = ArrayString::new();
    write!(&mut buf, "{}", s).expect("!write");
    buf
}
