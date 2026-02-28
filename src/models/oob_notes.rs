use thiserror::Error;
use base64::{engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD}, Engine};
use bech32::primitives::decode::CheckedHrpstring;
use k256::ecdsa::SigningKey;

use super::Note;

#[derive(Debug, Error)]
pub enum OobNotesError {
    #[error("Failed to decode ecash: {0}")]
    DecodeError(String),
    #[error("No notes found in ecash string")]
    NoNotes,
}

#[derive(Debug, Clone)]
pub struct ParsedNoteSet {
    pub federation_id: String,
    pub notes: Vec<Note>,
}

/// BigSize decoder (Lightning Network style variable length integer)
/// Note: BigSize uses BIG-ENDIAN encoding for multi-byte values
fn read_bigsize(data: &[u8], pos: &mut usize) -> Result<u64, OobNotesError> {
    if *pos >= data.len() {
        return Err(OobNotesError::DecodeError("Unexpected end of data reading BigSize".into()));
    }

    let first = data[*pos];
    *pos += 1;

    match first {
        0..=0xFC => Ok(first as u64),
        0xFD => {
            if *pos + 2 > data.len() {
                return Err(OobNotesError::DecodeError("Unexpected end of data reading u16".into()));
            }
            let val = u16::from_be_bytes([data[*pos], data[*pos + 1]]);
            *pos += 2;
            Ok(val as u64)
        }
        0xFE => {
            if *pos + 4 > data.len() {
                return Err(OobNotesError::DecodeError("Unexpected end of data reading u32".into()));
            }
            let val = u32::from_be_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
            *pos += 4;
            Ok(val as u64)
        }
        0xFF => {
            if *pos + 8 > data.len() {
                return Err(OobNotesError::DecodeError("Unexpected end of data reading u64".into()));
            }
            let val = u64::from_be_bytes([
                data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3],
                data[*pos + 4], data[*pos + 5], data[*pos + 6], data[*pos + 7],
            ]);
            *pos += 8;
            Ok(val)
        }
    }
}

/// Read a fixed number of bytes
fn read_bytes(data: &[u8], pos: &mut usize, count: usize) -> Result<Vec<u8>, OobNotesError> {
    if *pos + count > data.len() {
        return Err(OobNotesError::DecodeError(format!(
            "Unexpected end of data reading {} bytes at position {}",
            count, *pos
        )));
    }
    let result = data[*pos..*pos + count].to_vec();
    *pos += count;
    Ok(result)
}

/// Read a length-prefixed byte array
fn read_vec_bytes(data: &[u8], pos: &mut usize) -> Result<Vec<u8>, OobNotesError> {
    let len = read_bigsize(data, pos)? as usize;
    read_bytes(data, pos, len)
}

/// SpendableNote structure:
/// - signature: BLS G1 point (48 bytes compressed)
/// - spend_key: secp256k1 keypair (32 bytes secret key)
///
/// The nonce is the secp256k1 public key derived from the spend_key
const SIGNATURE_SIZE: usize = 48; // BLS G1 compressed
const SPEND_KEY_SIZE: usize = 32; // secp256k1 secret key

fn parse_spendable_note(data: &[u8], pos: &mut usize) -> Result<String, OobNotesError> {
    // Read signature (BLS G1 point - 48 bytes)
    let _signature = read_bytes(data, pos, SIGNATURE_SIZE)?;

    // Read spend_key (secp256k1 secret key - 32 bytes)
    let spend_key_bytes = read_bytes(data, pos, SPEND_KEY_SIZE)?;

    // Derive the secp256k1 public key from the secret key
    let signing_key = SigningKey::from_slice(&spend_key_bytes)
        .map_err(|e| OobNotesError::DecodeError(format!("Invalid spend_key: {}", e)))?;
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_sec1_bytes();

    // The nonce is the hex-encoded public key
    Ok(hex::encode(&public_key_bytes))
}

/// Parse TieredMulti<SpendableNote>: BTreeMap<Amount, Vec<SpendableNote>>
fn parse_tiered_multi_notes(data: &[u8], pos: &mut usize) -> Result<Vec<(u64, String)>, OobNotesError> {
    let mut notes = Vec::new();

    // Read map length
    let map_len = read_bigsize(data, pos)?;

    for _ in 0..map_len {
        // Read amount (msats as BigSize)
        let amount_msat = read_bigsize(data, pos)?;

        // Read Vec<SpendableNote> length
        let vec_len = read_bigsize(data, pos)?;

        for _ in 0..vec_len {
            let nonce = parse_spendable_note(data, pos)?;
            notes.push((amount_msat, nonce));
        }
    }

    Ok(notes)
}

/// OOBNotesPart variants (fedimint consensus encoding):
/// Each variant is encoded as: variant_index (BigSize) + data_length (BigSize) + data
///
/// 0: Notes(TieredMulti<SpendableNote>)
/// 1: FederationIdPrefix([u8; 4])
/// 2: Invite { peer_apis: Vec<(PeerId, SafeUrl)>, federation_id: FederationId }
/// 3: ApiSecret(String)
/// _: Default { variant, bytes }
fn parse_oob_notes_part(
    data: &[u8],
    pos: &mut usize,
) -> Result<Option<(Option<String>, Vec<(u64, String)>)>, OobNotesError> {
    let variant = read_bigsize(data, pos)?;
    // Read the length prefix for the variant data
    let data_len = read_bigsize(data, pos)? as usize;
    let data_end = *pos + data_len;

    if data_end > data.len() {
        return Err(OobNotesError::DecodeError(format!(
            "Variant data length {} exceeds remaining data at position {}",
            data_len, *pos
        )));
    }

    let result = match variant {
        0 => {
            // Notes(TieredMulti<SpendableNote>)
            let notes = parse_tiered_multi_notes(data, pos)?;
            Ok(Some((None, notes)))
        }
        1 => {
            // FederationIdPrefix([u8; 4])
            let prefix = read_bytes(data, pos, 4)?;
            let fed_id = hex::encode(&prefix);
            Ok(Some((Some(fed_id), vec![])))
        }
        2 => {
            // Invite { peer_apis, federation_id }
            // Read Vec<(PeerId, SafeUrl)>
            let peer_apis_len = read_bigsize(data, pos)?;
            for _ in 0..peer_apis_len {
                // PeerId is u16
                let _peer_id = read_bigsize(data, pos)?;
                // SafeUrl is length-prefixed string
                let _url = read_vec_bytes(data, pos)?;
            }
            // FederationId is 32 bytes
            let fed_id_bytes = read_bytes(data, pos, 32)?;
            let fed_id = hex::encode(&fed_id_bytes);
            Ok(Some((Some(fed_id), vec![])))
        }
        3 => {
            // ApiSecret(String)
            let _secret = read_vec_bytes(data, pos)?;
            Ok(Some((None, vec![])))
        }
        _ => {
            // Unknown variant - skip the data
            *pos = data_end;
            Ok(Some((None, vec![])))
        }
    };

    // Ensure we consumed exactly the expected amount of data
    if *pos != data_end {
        // If we didn't consume all data, skip to the end
        *pos = data_end;
    }

    result
}

/// Decode the raw bytes of OOBNotes (after base64/bech32 decoding)
fn decode_oob_notes_bytes(data: &[u8]) -> Result<ParsedNoteSet, OobNotesError> {
    let mut pos = 0;
    let mut federation_id: Option<String> = None;
    let mut all_notes: Vec<(u64, String)> = Vec::new();

    // Read Vec<OOBNotesPart> length
    let parts_len = read_bigsize(data, &mut pos)?;

    for _ in 0..parts_len {
        if let Some((fed_id, notes)) = parse_oob_notes_part(data, &mut pos)? {
            if fed_id.is_some() {
                federation_id = fed_id;
            }
            all_notes.extend(notes);
        }
    }

    if all_notes.is_empty() {
        return Err(OobNotesError::NoNotes);
    }

    let federation_id = federation_id.unwrap_or_else(|| "unknown".to_string());

    let notes = all_notes
        .into_iter()
        .map(|(amount_msat, nonce)| Note::new(nonce, amount_msat))
        .collect();

    Ok(ParsedNoteSet {
        federation_id,
        notes,
    })
}

/// Try to decode base64 with multiple variants
fn try_decode_base64(input: &str) -> Option<Vec<u8>> {
    // Try URL-safe without padding first (most common for ecash)
    if let Ok(bytes) = URL_SAFE_NO_PAD.decode(input) {
        return Some(bytes);
    }
    // Try URL-safe with padding
    if let Ok(bytes) = URL_SAFE.decode(input) {
        return Some(bytes);
    }
    // Try standard base64 with padding
    if let Ok(bytes) = STANDARD.decode(input) {
        return Some(bytes);
    }
    // Try standard without padding
    let standard_no_pad = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::NO_PAD,
    );
    if let Ok(bytes) = standard_no_pad.decode(input) {
        return Some(bytes);
    }
    None
}

/// Parse OOBNotes from a string.
/// Supports:
/// - Bech32 format with "fedimint1" prefix
/// - Base64 URL-safe (with or without padding)
/// - Standard base64 (with or without padding)
pub fn parse_oob_notes(input: &str) -> Result<ParsedNoteSet, OobNotesError> {
    // Remove whitespace
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    if input.is_empty() {
        return Err(OobNotesError::DecodeError("Empty input".to_string()));
    }

    // Try bech32 first (starts with fedimint1)
    let bytes = if input.to_lowercase().starts_with("fedimint1") {
        // Bech32 decode
        let checked = CheckedHrpstring::new::<bech32::Bech32m>(&input)
            .map_err(|e| OobNotesError::DecodeError(format!("Invalid bech32: {}", e)))?;

        // Check HRP
        if checked.hrp().as_str() != "fedimint" {
            return Err(OobNotesError::DecodeError("Invalid HRP, expected 'fedimint'".into()));
        }

        // Extract data bytes (byte_iter already converts from 5-bit to 8-bit)
        checked.byte_iter().collect()
    } else {
        // Try base64 variants
        try_decode_base64(&input)
            .ok_or_else(|| OobNotesError::DecodeError("Invalid base64 encoding".to_string()))?
    };

    decode_oob_notes_bytes(&bytes)
}

/// Parses multiple ecash strings (e.g., from CSV, one per line)
pub fn parse_csv_notes(csv_content: &str) -> Vec<Result<ParsedNoteSet, OobNotesError>> {
    csv_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(parse_oob_notes)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_input() {
        let result = parse_oob_notes("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_input() {
        let result = parse_oob_notes("not-valid-ecash");
        assert!(result.is_err());
    }

    #[test]
    fn test_bigsize_decode() {
        // Test single byte values
        let data = [0x00, 0x7F, 0xFC];
        let mut pos = 0;
        assert_eq!(read_bigsize(&data, &mut pos).unwrap(), 0);
        assert_eq!(read_bigsize(&data, &mut pos).unwrap(), 0x7F);
        assert_eq!(read_bigsize(&data, &mut pos).unwrap(), 0xFC);

        // Test 2-byte value (big-endian: 0x0100 = 256)
        let data = [0xFD, 0x01, 0x00];
        let mut pos = 0;
        assert_eq!(read_bigsize(&data, &mut pos).unwrap(), 256);

        // Test 4-byte value (big-endian: 0x00100000 = 1048576)
        let data = [0xFE, 0x00, 0x10, 0x00, 0x00];
        let mut pos = 0;
        assert_eq!(read_bigsize(&data, &mut pos).unwrap(), 1048576);
    }
}
