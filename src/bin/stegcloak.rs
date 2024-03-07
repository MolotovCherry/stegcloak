fn main() {
    let encrypt = stegcloak::encrypt::hide(
        "mysecret a perfectly normal message without any repeating and stuff like that",
        "mypassword",
        false,
        "cover text",
    )
    .unwrap();

    let decrypt = stegcloak::encrypt::reveal("mypassword", false, encrypt).unwrap();

    println!("{decrypt:?}");
}
