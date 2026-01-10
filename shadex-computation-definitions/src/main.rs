fn main() {
    let mut unit = wgsl_parse::parse_str(include_str!("../test.wgsl")).unwrap();

    println!("{:?}", unit);
}
