use stegcloak::encrypt;
use stegcloak::plaintext;

fn main() {
    // plaintext is fairly easy
    let hidden = plaintext::hide("mysecret", "cover text").unwrap();
    let secret = plaintext::reveal(hidden).unwrap();
    assert_eq!("mysecret", secret);

    // encrypt. You can skip the integrity check
    let hidden = encrypt::hide("mysecret", "password", false, "cover text").unwrap();
    let secret = encrypt::reveal("password", false, hidden).unwrap();
    assert_eq!("mysecret", secret);

    // encrypt. But with the data integrity check for maximum security
    let hidden = encrypt::hide("mysecret", "password", true, "cover text").unwrap();
    let secret = encrypt::reveal("password", true, hidden).unwrap();
    assert_eq!("mysecret", secret);
}
