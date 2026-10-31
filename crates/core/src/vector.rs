//! Vector helpers: f32 <-> BLOB codec and brute-force cosine similarity.
//! The optional `sqlite-vec` index uses the same encoding.

pub fn to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub fn from_blob(b: &[u8]) -> Vec<f32> {
    b.as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect()
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_and_blob_roundtrip() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [1.0f32, 2.0, 3.0];
        assert!((cosine(&a, &b) - 1.0).abs() < 1e-6);
        assert!((cosine(&a, &[-1.0, -2.0, -3.0]) + 1.0).abs() < 1e-6);
        assert_eq!(from_blob(&to_blob(&a)), a.to_vec());
    }
}
