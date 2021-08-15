use std::convert::TryInto;

fn quarter_round(sequence: &mut [u32; 4]) {
    let shifts = [7, 9, 13, 18];

    for (i, &shift) in shifts.iter().enumerate() {
        sequence[(i + 1) % sequence.len()] ^= sequence[i]
            .wrapping_add(sequence[(i + 3) % sequence.len()])
            .rotate_left(shift);
    }
}

fn generic_round(sequence: &mut [u32; 16], index_array: [[usize; 4]; 4]) {
    for indicies in index_array {
        let mut mod_seq = [
            sequence[indicies[0]],
            sequence[indicies[1]],
            sequence[indicies[2]],
            sequence[indicies[3]],
        ];
        quarter_round(&mut mod_seq);

        for (i, &j) in indicies.iter().enumerate() {
            sequence[j] = mod_seq[i];
        }
    }
}

fn row_round(sequence: &mut [u32; 16]) {
    let index_array = [[0, 1, 2, 3], [5, 6, 7, 4], [10, 11, 8, 9], [15, 12, 13, 14]];

    generic_round(sequence, index_array)
}

fn column_round(sequence: &mut [u32; 16]) {
    let index_array = [[0, 4, 8, 12], [5, 9, 13, 1], [10, 14, 2, 6], [15, 3, 7, 11]];

    generic_round(sequence, index_array)
}

fn double_round(sequence: &mut [u32; 16]) {
    column_round(sequence);
    row_round(sequence);
}

fn hash(sequence: [u8; 64]) -> [u8; 64] {
    let mut words = [0_u32; 16];

    for (i, bytes) in sequence.chunks(4).enumerate() {
        words[i] = u32::from_le_bytes(bytes.try_into().unwrap());
    }

    let mut words_copy = words;

    for _round in 0..10 {
        double_round(&mut words_copy);
    }

    let mut sequence = [0_u8; 64];

    for (i, (&word_x, &word_z)) in words.iter().zip(words_copy.iter()).enumerate() {
        let word = word_x.wrapping_add(word_z);
        let bytes = u32::to_le_bytes(word);

        for (j, &byte) in bytes.iter().enumerate() {
            sequence[i * 4 + j] = byte;
        }
    }

    sequence
}

fn expand_32(key: [u8; 32], nonce: [u8; 8], index: u64) -> [u8; 64] {
    const CONSTANTS: [[u8; 4]; 4] = [
        [101_u8, 120, 112, 97],
        [110, 100, 32, 51],
        [50, 45, 98, 121],
        [116, 101, 32, 107],
    ];

    let mut sequence = [0_u8; 64];

    sequence[0..4].copy_from_slice(&CONSTANTS[0]);
    sequence[4..20].copy_from_slice(&key[0..16]);
    sequence[20..24].copy_from_slice(&CONSTANTS[1]);
    sequence[24..32].copy_from_slice(&nonce);
    sequence[32..40].copy_from_slice(&index.to_le_bytes());
    sequence[40..44].copy_from_slice(&CONSTANTS[2]);
    sequence[44..60].copy_from_slice(&key[16..32]);
    sequence[60..64].copy_from_slice(&CONSTANTS[3]);

    hash(sequence)
}

fn expand_16(key: [u8; 16], nonce: [u8; 8], index: u64) -> [u8; 64] {
    const CONSTANTS: [[u8; 4]; 4] = [
        [101_u8, 120, 112, 97],
        [110, 100, 32, 49],
        [54, 45, 98, 121],
        [116, 101, 32, 107],
    ];

    let mut sequence = [0_u8; 64];

    sequence[..4].copy_from_slice(&CONSTANTS[0]);
    sequence[4..20].copy_from_slice(&key);
    sequence[20..24].copy_from_slice(&CONSTANTS[1]);
    sequence[24..32].copy_from_slice(&nonce);
    sequence[32..40].copy_from_slice(&index.to_le_bytes());
    sequence[40..44].copy_from_slice(&CONSTANTS[2]);
    sequence[44..60].copy_from_slice(&key);
    sequence[60..].copy_from_slice(&CONSTANTS[3]);

    hash(sequence)
}

pub fn encrypt_decrypt_32(key: [u8; 32], nonce: [u8; 8], buf: &mut [u8]) {
    for (index, block) in buf.chunks_mut(64).enumerate() {
        let key_seq = expand_32(key, nonce, index as u64);

        for (i, byte) in block.iter_mut().enumerate() {
            *byte ^= key_seq[i];
        }
    }
}

pub fn encrypt_decrypt_16(key: [u8; 16], nonce: [u8; 8], buf: &mut [u8]) {
    for (index, block) in buf.chunks_mut(64).enumerate() {
        let key_seq = expand_16(key, nonce, index as u64);

        for (i, byte) in block.iter_mut().enumerate() {
            *byte ^= key_seq[i];
        }
    }
}

#[cfg(test)]
mod test;
