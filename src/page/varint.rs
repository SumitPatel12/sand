use super::errors::DBError;
use std::convert::TryInto;

// NOTE: Seems like this is someting different. I ended up using this for the first pass. It gave
// the right answers until it didn't. Leaving in for reference in the future. LINK TO THE OFFENDING
// DOC and my mistake for not understanding it: https://sqlite.org/src4/doc/trunk/www/varint.wiki#:~:text=SQLite4%3A%20Variable%2DLength%20Integers,for%20varints%20are%20the%20same.
pub fn get_varint_incorrect_version(buffer: &[u8]) -> Result<(u64, usize), DBError> {
    match buffer[0] {
        x if x <= 240 => return Ok((buffer[0] as u64, 1)),
        x if x <= 248 => return Ok((240 + 256 * (buffer[0] as u64 - 241) + buffer[1] as u64, 2)),
        x if x == 249 => return Ok((2288 + 256 * buffer[1] as u64 + buffer[2] as u64, 3)),
        x if x == 250 => {
            return Ok((read_u64_from_bytes(&buffer[1..4]), 4));
        }
        x if x == 251 => {
            return Ok((read_u64_from_bytes(&buffer[1..5]), 5));
        }
        x if x == 252 => {
            return Ok((read_u64_from_bytes(&buffer[1..6]), 6));
        }
        x if x == 253 => {
            return Ok((read_u64_from_bytes(&buffer[1..7]), 7));
        }
        x if x == 254 => {
            return Ok((read_u64_from_bytes(&buffer[1..8]), 8));
        }
        x if x == 255 => {
            return Ok((
                u64::from_be_bytes((&buffer[1..9]).try_into().expect("Incorrect Length")),
                9,
            ));
        }
        _ => return Err(DBError::InvalidVarintSize),
    };
}

pub fn get_varint(buffer: &[u8], offset: &mut usize) -> u64 {
    // TODO: Add logic to handle case when the lenght is 9.
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

    #[test]
    fn get_varint_test_size_1() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0x82, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 130);
        assert_eq!(size, 1);
    }

    #[test]
    fn get_varint_test_size_2() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xF1, 0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 275);
        assert_eq!(size, 2);
    }

    #[test]
    fn get_varint_test_size_3() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xF9, 0x0A, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 4859);
        assert_eq!(size, 3);
    }

    #[test]
    fn get_varint_test_size_4() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xFA, 0x12, 0x34, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 1193046);
        assert_eq!(size, 4);
    }

    #[test]
    fn get_varint_test_size_5() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xFB, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 305419896);
        assert_eq!(size, 5);
    }

    #[test]
    fn get_varint_test_size_6() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xFC, 0x01, 0x23, 0x45, 0x67, 0x89, 0x00, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 4886718345);
        assert_eq!(size, 6);
    }

    #[test]
    fn get_varint_test_size_7() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xFD, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0x00, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 1250999896491);
        assert_eq!(size, 7);
    }

    #[test]
    fn get_varint_test_size_8() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xFE, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0x00])
                .unwrap();
        assert_eq!(decoded_varint, 320255973501901);
        assert_eq!(size, 8);
    }

    #[test]
    fn get_varint_test_size_9() {
        let (decoded_varint, size) =
            get_varint_incorrect_version(&[0xFF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF])
                .unwrap();
        assert_eq!(decoded_varint, 81985529216486895);
        assert_eq!(size, 9);
    }
}
