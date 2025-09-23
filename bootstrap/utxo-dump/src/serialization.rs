use std::io::{Error, ErrorKind, Read};

/// Decompress amount value (Bitcoin Core compression)
pub(crate) fn decompress_amount(compressed: u64) -> Result<u64, Error> {
    if compressed == 0 {
        return Ok(0);
    }

    let mut x = compressed - 1;

    let e = (x % 10) as u32; // remainder mod 10
    x = x / 10; // quotient mod 10 (reduce x down by 10)

    // If the remainder is less than 9
    let n = if e < 9 {
        let d = x % 9 + 1;
        x = x / 9;
        x * 10 + d
    } else {
        x + 1
    };

    let result = n.checked_mul(10u64.pow(e))
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Amount overflow during decompression"))?;

    Ok(result)
}

/// Read raw varint bytes using Bitcoin's custom format
/// Ref: <https://github.com/bitcoin/bitcoin/blob/aa87e0b44600a32b32a4b123d4f90d097f1f106f/src/serialize.h#L452>
pub(crate) fn read_varint<R: Read>(reader: &mut R) -> Result<u64, Error> {
    let mut n = 0u64;

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let ch_data = byte[0];

        if n > (u64::MAX >> 7) {
            return Err(Error::new(ErrorKind::InvalidData, "read varint overflow"));
        }

        n = (n << 7) | ((ch_data & 0x7F) as u64);

        if (ch_data & 0x80) != 0 {
            // High bit set: increment n and continue
            n = n.checked_add(1)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "read varint overflow"))?;
        } else {
            // High bit clear: this is the last byte
            return Ok(n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper function to test read_varint with byte arrays
    fn test_read_varint(bytes: &[u8]) -> Result<u64, Error> {
        let mut cursor = Cursor::new(bytes);
        read_varint(&mut cursor)
    }

    #[test]
    fn test_read_varint_single_byte() {
        assert_eq!(test_read_varint(&[0x00]).unwrap(), 0);
        assert_eq!(test_read_varint(&[0x01]).unwrap(), 1);
        assert_eq!(test_read_varint(&[0x7F]).unwrap(), 127);
    }

    #[test] 
    fn test_read_varint_multi_byte() {
        assert_eq!(test_read_varint(&[0x80, 0x00]).unwrap(), 128);
        assert_eq!(test_read_varint(&[0x80, 0x01]).unwrap(), 129);
        assert_eq!(test_read_varint(&[0x81, 0x00]).unwrap(), 256);
        assert_eq!(test_read_varint(&[0xFF, 0x7F]).unwrap(), 16511);
        
        assert_eq!(test_read_varint(&[0x80, 0x80, 0x00]).unwrap(), 16512);
        assert_eq!(test_read_varint(&[0x82, 0x84, 0x7E]).unwrap(), 49918);
    }

    #[test]
    fn test_read_varint_bitcoin_examples() {
        // Example from: <https://github.com/bitcoin/bitcoin/blob/8d801e3efbf1e3b1f9a0060b777788f271cb21c9/src/test/streams_tests.cpp#L179>
        assert_eq!(test_read_varint(&[0x82, 0xA7, 0x31]).unwrap(), 54321);
        // Example from: https://github.com/dogecoin/dogecoin/blob/265f258540ed36982a43ba38f55b5f3558f0bf74/src/test/coins_tests.cpp#L491
        assert_eq!(test_read_varint(&[0x8A, 0x95, 0xC0, 0xBB, 0x00]).unwrap(), 3000000000)
    }

    #[test]
    fn test_read_varint_edge_cases() {
        assert!(test_read_varint(&[]).is_err());
        
        // Incomplete varint (high bit set but no following byte)
        assert!(test_read_varint(&[0x80]).is_err());
    }

    #[test]
    fn test_read_varint_overflow() {
        // Test overflow protection
        // Create a varint that would exceed u64::MAX
        let overflow_bytes = vec![0xFF; 10];
        assert!(test_read_varint(&overflow_bytes).is_err());
        
        // Test the boundary condition n > (u64::MAX >> 7)
        let boundary_bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert!(test_read_varint(&boundary_bytes).is_err());
    }

    #[test]
    fn test_decompress_amount_zero() {
        assert_eq!(decompress_amount(0).unwrap(), 0);
    }

    #[test]
    fn test_decompress_amount_small_values() {
        // Test basic decompression for small amounts
        assert_eq!(decompress_amount(1).unwrap(), 1);
        assert_eq!(decompress_amount(2).unwrap(), 10);
        assert_eq!(decompress_amount(3).unwrap(), 100);
        assert_eq!(decompress_amount(4).unwrap(), 1000);
        assert_eq!(decompress_amount(5).unwrap(), 10000);
    }

    #[test]
    fn test_decompress_amount_pattern_e_less_than_9() {
        // Test cases where e < 9 (first compression pattern)
        // Algorithm: compressed -> x = compressed-1 -> e = x%10, x = x/10 -> d = x%9+1, x = x/9 -> n = x*10+d -> result = n * 10^e
        assert_eq!(decompress_amount(11).unwrap(), 2);     // x=10->1, e=0, d=2, x=0, n=2, result=2*10^0=2  
        assert_eq!(decompress_amount(21).unwrap(), 3);     // x=20->2, e=0, d=3, x=0, n=3, result=3*10^0=3
        assert_eq!(decompress_amount(12).unwrap(), 20);    // x=11->1, e=1, d=2, x=0, n=2, result=2*10^1=20
        assert_eq!(decompress_amount(13).unwrap(), 200);   // x=12->1, e=2, d=2, x=0, n=2, result=2*10^2=200
        assert_eq!(decompress_amount(31).unwrap(), 4);     // x=30->3, e=0, d=4, x=0, n=4, result=4*10^0=4
        assert_eq!(decompress_amount(22).unwrap(), 30);    // x=21->2, e=1, d=3, x=0, n=3, result=3*10^1=30
        assert_eq!(decompress_amount(91).unwrap(), 11);    // x=90->9, e=0, d=1, x=1, n=11, result=11*10^0=11
    }

    #[test]
    fn test_decompress_amount_pattern_e_equals_9() {
        // Test cases where e = 9 (second compression pattern)
        // For e = 9, we need x % 10 = 9, so compressed values ending in 0 (since x = compressed - 1)
        assert_eq!(decompress_amount(10).unwrap(), 1000000000);  // x=9, e=9, x=0, n=1, result=1*10^9=1000000000
        assert_eq!(decompress_amount(20).unwrap(), 2000000000);  // x=19, e=9, x=1, n=2, result=2*10^9=2000000000
        assert_eq!(decompress_amount(30).unwrap(), 3000000000);  // x=29, e=9, x=2, n=3, result=3*10^9=3000000000
        assert_eq!(decompress_amount(100).unwrap(), 10000000000); // x=99, e=9, x=9, n=10, result=10*10^9=10000000000
    }

    #[test]
    fn test_decompress_amount_bitcoin_examples() {
        // Examples from: <https://github.com/bitcoin/bitcoin/blob/fa0fe08eca48064b2a42789571fea017e455d820/src/test/compress_tests.cpp#L41>
        assert_eq!(decompress_amount(0x0).unwrap(), 0);
        assert_eq!(decompress_amount(0x1).unwrap(), 1);
        assert_eq!(decompress_amount(0x7).unwrap(), 1_000_000);   // 0.01 BTC in satoshis
        assert_eq!(decompress_amount(0x9).unwrap(), 100_000_000);   // 1 BTC in satoshis
        assert_eq!(decompress_amount(0x32).unwrap(), 50 * 100_000_000);   // 50 BTC in satoshis
        assert_eq!(decompress_amount(0x1406f40).unwrap(), 21_000_000 * 100_000_000);   // 21 M BTC in satoshis
    }

    #[test]
    fn test_decompress_amount_large_values() {
        assert_eq!(decompress_amount(100).unwrap(), 10000000000);   // x=99, e=9, x=9, n=10, result=10*10^9=10000000000
        assert_eq!(decompress_amount(500).unwrap(), 50000000000);   // x=499, e=9, x=49, n=50, result=50*10^9=50000000000
        
        // e < 9
        assert_eq!(decompress_amount(987).unwrap(), 109000000);     // x=986, e=6, x=98, d=9, x=10, n=109, result=109*10^6=109000000
        assert_eq!(decompress_amount(456).unwrap(), 5100000);       // x=455, e=5, x=45, d=1, x=5, n=51, result=51*10^5=5100000
    }
}