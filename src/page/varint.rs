/*
* Varint are variable length integer mainly used for space optimization. For sqlite they can be
* upto 9 bytes in size.
* They use Huffman encoding, the idea being that the first bit of the byte is used to indicate if
* we read one more byte. We repeat until 8 bytes and if the high bit of the 8th byte is set we use
* all bits of the 9th byte.
* We encode in big endian.
*
* For Example:
*   Say we want to encode 300 whose binary is 100101100.
*   This would be split into 10, 0101100. Which would then be encoded as
*   - Binary: 10000010 00101100
*   - Hex: 0x82 0x2C
*
*   To decode it we'd take the 7 least significant bits from both the bytes and combine them to get
*   the initial value.
*   - 00000100101100 => 100101100
*   This was our intial number.
*/
pub fn get_varint(buffer: &[u8], offset: &mut usize) -> u64 {
    let mut size = 0;
    let mut decoded_varint: u64 = 0;

    while size < 8 && buffer[*offset] >= 0x80 {
        decoded_varint |= u64::from(buffer[*offset]) & 0x7F;
        decoded_varint <<= 7;
        *offset += 1;
        size += 1;
    }

    decoded_varint |= u64::from(buffer[*offset]) & 0x7F;
    *offset += 1;
    decoded_varint
}

// TODO: Read documentation of other dbs for encoding and make a more generic function.
// Implement this as well
pub fn encode_varint(value: u64) -> Vec<u8> {
    vec![0u8; 1]
}

pub fn read_u64_from_bytes(bytes: &[u8]) -> u64 {
    // TODO: Add some handling for when the size is not between 3 and 7.
    let mut result: u64 = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        result |= (byte as u64) << (8 * (bytes.len() - 1 - i));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_varint_test() {
        let result = get_varint(&[0x82, 0x2C], &mut 0);
        assert_eq!(result, 300 as u64);

        let result = get_varint(&[0x81, 0x07], &mut 0);
        assert_eq!(result, 135 as u64);
    }
}
