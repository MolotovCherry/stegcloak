use stegcloak::codec;

fn main() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    // create pure zwc string which encodes the data
    let mut encoded = codec::encode(&data);
    // note, decoding requires a space before the zwc!
    encoded.insert(0, ' ');
    let decoded = codec::decode(&encoded).unwrap();

    assert_eq!(decoded, data);
}
