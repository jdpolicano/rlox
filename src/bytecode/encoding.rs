// This module defines a tightly packed line encoding for meta-data (line info, specifically)
// for our VM. It provides functionality to encode and decode line information efficiently.
// "Short" length constants are any constant that is between 0 and 2^7 (inclusive).
// "Long" length encodings can represent values up to 2^15. If the most significant bit (MSB)
// of the current byte being read is set, it indicates a long encoded index.
// The module will then read the following byte, combine the two bytes into a u16,
// and convert it to a usize. If the MSB is not set, the byte is interpreted as a single u8,
// cast to a usize, and returned.
use std::io::{self, Read, Write};

const MAX_SHORT_SIZE: usize = i8::MAX as usize; // the max size for short encoding.
const MAX_LONG_SIZE: usize = i16::MAX as usize; // the max size for long encoding.
const MSB: u8 = 0x80; // a mask for the most significant bit.

pub struct SizeEncodedVec {
    sizes: Vec<u8>,
    // pointer to the beginning of the last element so that we can reliably read the last encoded value
    tail: usize,
}

impl SizeEncodedVec {
    /// Creates a new, empty `SizeEncodedVec`.
    pub fn new() -> Self {
        Self {
            sizes: Vec::new(),
            tail: 0,
        }
    }

    /// Encodes a usize value and appends it to the internal vector.
    pub fn push(&mut self, value: usize) -> io::Result<()> {
        let start_len = self.sizes.len();
        if let Err(e) = encode_usize(value, &mut self.sizes) {
            self.sizes.truncate(start_len); // Roll back any partial writes on error.
            return Err(e);
        }
        self.tail = start_len;
        Ok(())
    }

    /// Decodes and removes the next usize value from the internal vector's front half.
    /// Returns `None` if the vector is empty or contains invalid data.
    pub fn shift(&mut self) -> Option<io::Result<usize>> {
        if self.sizes.is_empty() {
            return None;
        }

        let mut cursor = &self.sizes[..];
        match decode_usize(&mut cursor) {
            Ok(value) => {
                let consumed = self.sizes.len() - cursor.len();
                self.sizes.drain(0..consumed);
                self.tail = if self.sizes.is_empty() {
                    0
                } else {
                    self.tail - consumed
                };
                Some(Ok(value))
            }
            Err(e) => Some(Err(e)),
        }
    }

    pub fn peek_last(&self) -> Option<io::Result<usize>> {
        if self.sizes.is_empty() {
            return None;
        }
        let mut cursor = &self.sizes[self.tail..];
        Some(decode_usize(&mut cursor))
    }

    /// Returns the number of bytes currently stored in the internal vector.
    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    /// Returns `true` if the internal vector is empty.
    pub fn is_empty(&self) -> bool {
        self.sizes.is_empty()
    }
}

/// Encodes a usize into a compact byte representation.
/// If the value is less than or equal to 2^7, it is encoded as a single byte.
/// If the value is greater than 2^7, it is encoded as two bytes with the MSB of the first byte set.
pub fn encode_usize(value: usize, writer: &mut impl Write) -> io::Result<()> {
    // If the value is less than or equal to MAX_SHORT_SIZE (127), it can be encoded in a single byte.
    if value <= MAX_SHORT_SIZE {
        writer.write_all(&[value as u8])?; // Write the value directly as a single byte.
    }
    // If the value is greater than MAX_SHORT_SIZE but less than or equal to MAX_LONG_SIZE (32,767),
    // it needs to be encoded in two bytes. The first byte's MSB is set to indicate a long encoding.
    else if value <= MAX_LONG_SIZE {
        let high_byte = ((value >> 8) as u8) | MSB; // Extract the high 7 bits and set the MSB.
        let low_byte = (value & 0xFF) as u8; // Extract the low 8 bits.
        writer.write_all(&[high_byte, low_byte])?; // Write the two bytes to the writer.
    }
    // If the value exceeds MAX_LONG_SIZE, it cannot be encoded using this scheme.
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Value too large to encode", // Return an error indicating the value is too large.
        ));
    }
    Ok(())
}

/// Decodes a usize from a compact byte representation.
/// Reads one or two bytes depending on the MSB of the first byte.
pub fn decode_usize(reader: &mut impl Read) -> io::Result<usize> {
    let mut first_byte = [0u8]; // Buffer to store the first byte read from the input.
    reader.read_exact(&mut first_byte)?; // Read the first byte from the reader.

    // Check if the MSB of the first byte is not set (indicating a short encoding).
    if first_byte[0] & MSB == 0 {
        // If the MSB is not set, the value is a single byte and can be directly cast to usize.
        Ok(first_byte[0] as usize)
    } else {
        // If the MSB is set, it indicates a long encoding, requiring a second byte.
        let mut second_byte = [0u8]; // Buffer to store the second byte.
        reader.read_exact(&mut second_byte)?; // Read the second byte from the reader.
        // Extract the high 7 bits from the first byte (ignoring the MSB).
        let high_part = (first_byte[0] & 0x7F) as usize;
        // Extract the full 8 bits from the second byte.
        let low_part = second_byte[0] as usize;
        // Combine the high and low parts to reconstruct the original usize value.
        Ok((high_part << 8) | low_part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_short() {
        let mut buffer = Vec::new();
        encode_usize(127, &mut buffer).unwrap();
        assert_eq!(buffer, vec![127]);

        let mut cursor = &buffer[..];
        let decoded = decode_usize(&mut cursor).unwrap();
        assert_eq!(decoded, 127);
    }

    #[test]
    fn test_encode_decode_long() {
        let mut buffer = Vec::new();
        encode_usize(300, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81, 0x2C]);

        let mut cursor = &buffer[..];
        let decoded = decode_usize(&mut cursor).unwrap();
        assert_eq!(decoded, 300);
    }

    #[test]
    fn test_encode_too_large() {
        let mut buffer = Vec::new();
        let result = encode_usize(0x1_0000, &mut buffer);
        assert!(result.is_err());
    }
}
