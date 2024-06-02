

pub fn consistent_hash<T : AsRef<[u8]>>(v: T) -> u64 {
    let mut hash = 5381_u64;

    for c in v.as_ref() {
        hash = hash.wrapping_mul(33_u64.wrapping_pow(*c as u32));
    }

    return hash;
}