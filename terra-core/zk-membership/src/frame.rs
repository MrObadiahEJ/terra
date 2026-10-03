use ark_bn254::{Bn254, Fq, G1Affine, G2Affine};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Proof, VerifyingKey};
use ark_serialize::Valid;

pub const TG16_MAGIC: [u8; 4] = *b"TG16";
pub const TG16_VERSION: u8 = 1;
pub const TG16_SIGNATURE_LEN: usize = 64;
pub const TG16_VK_LEN: usize = 576;
pub const TG16_PROOF_LEN: usize = 256;
pub const TG16_FRAME_LEN: usize = 4 + 1 + TG16_SIGNATURE_LEN + TG16_VK_LEN + TG16_PROOF_LEN;

#[derive(Clone)]
pub struct Frame {
    pub signature: [u8; TG16_SIGNATURE_LEN],
    pub verification_key: [u8; TG16_VK_LEN],
    pub proof: [u8; TG16_PROOF_LEN],
}

pub fn decode_frame(data: &[u8]) -> Result<Frame, &'static str> {
    if data.len() != TG16_FRAME_LEN {
        return Err("proof frame must be exactly 901 bytes");
    }
    if data[..4] != TG16_MAGIC {
        return Err("proof frame magic is invalid");
    }
    if data[4] != TG16_VERSION {
        return Err("proof frame version is unsupported");
    }
    let mut signature = [0; TG16_SIGNATURE_LEN];
    let mut verification_key = [0; TG16_VK_LEN];
    let mut proof = [0; TG16_PROOF_LEN];
    signature.copy_from_slice(&data[5..69]);
    verification_key.copy_from_slice(&data[69..645]);
    proof.copy_from_slice(&data[645..901]);
    Ok(Frame {
        signature,
        verification_key,
        proof,
    })
}

pub fn encode_frame(
    signature: &[u8; TG16_SIGNATURE_LEN],
    verification_key: &VerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
) -> Result<Vec<u8>, &'static str> {
    let vk_bytes = serialize_verification_key(verification_key)?;
    let proof_bytes = serialize_proof(proof);
    let mut frame = Vec::with_capacity(TG16_FRAME_LEN);
    frame.extend_from_slice(&TG16_MAGIC);
    frame.push(TG16_VERSION);
    frame.extend_from_slice(signature);
    frame.extend_from_slice(&vk_bytes);
    frame.extend_from_slice(&proof_bytes);
    Ok(frame)
}

pub fn serialize_verification_key(
    vk: &VerifyingKey<Bn254>,
) -> Result<[u8; TG16_VK_LEN], &'static str> {
    if vk.gamma_abc_g1.len() != 2 {
        return Err("the verifier expects exactly one public Groth16 input");
    }
    let mut out = [0; TG16_VK_LEN];
    out[0..64].copy_from_slice(&g1_be(&vk.alpha_g1));
    out[64..192].copy_from_slice(&g2_be(&vk.beta_g2));
    out[192..320].copy_from_slice(&g2_be(&vk.gamma_g2));
    out[320..448].copy_from_slice(&g2_be(&vk.delta_g2));
    out[448..512].copy_from_slice(&g1_be(&vk.gamma_abc_g1[0]));
    out[512..576].copy_from_slice(&g1_be(&vk.gamma_abc_g1[1]));
    Ok(out)
}

pub fn parse_verification_key(
    bytes: &[u8; TG16_VK_LEN],
) -> Result<VerifyingKey<Bn254>, &'static str> {
    let vk = VerifyingKey {
        alpha_g1: parse_g1(&bytes[0..64])?,
        beta_g2: parse_g2(&bytes[64..192])?,
        gamma_g2: parse_g2(&bytes[192..320])?,
        delta_g2: parse_g2(&bytes[320..448])?,
        gamma_abc_g1: vec![parse_g1(&bytes[448..512])?, parse_g1(&bytes[512..576])?],
    };
    vk.check()
        .map_err(|_| "verification key contains invalid curve points")?;
    Ok(vk)
}

pub fn parse_proof(bytes: &[u8; TG16_PROOF_LEN]) -> Result<Proof<Bn254>, &'static str> {
    let proof = Proof {
        a: parse_g1(&bytes[0..64])?,
        b: parse_g2(&bytes[64..192])?,
        c: parse_g1(&bytes[192..256])?,
    };
    proof
        .check()
        .map_err(|_| "proof contains invalid curve points")?;
    Ok(proof)
}

fn serialize_proof(proof: &Proof<Bn254>) -> [u8; TG16_PROOF_LEN] {
    let mut out = [0; TG16_PROOF_LEN];
    out[0..64].copy_from_slice(&g1_be(&proof.a));
    out[64..192].copy_from_slice(&g2_be(&proof.b));
    out[192..256].copy_from_slice(&g1_be(&proof.c));
    out
}

fn fq_be(value: &Fq) -> [u8; 32] {
    let bytes = value.into_bigint().to_bytes_be();
    let mut out = [0; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}

fn g1_be(point: &G1Affine) -> [u8; 64] {
    let mut out = [0; 64];
    out[..32].copy_from_slice(&fq_be(&point.x));
    out[32..].copy_from_slice(&fq_be(&point.y));
    out
}

fn g2_be(point: &G2Affine) -> [u8; 128] {
    let mut out = [0; 128];
    out[..32].copy_from_slice(&fq_be(&point.x.c1));
    out[32..64].copy_from_slice(&fq_be(&point.x.c0));
    out[64..96].copy_from_slice(&fq_be(&point.y.c1));
    out[96..].copy_from_slice(&fq_be(&point.y.c0));
    out
}

fn parse_fq(bytes: &[u8]) -> Result<Fq, &'static str> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| "invalid coordinate length")?;
    let field = Fq::from_be_bytes_mod_order(&bytes);
    let encoded = field.into_bigint().to_bytes_be();
    let mut canonical = [0; 32];
    canonical[32 - encoded.len()..].copy_from_slice(&encoded);
    if canonical != bytes {
        return Err("curve coordinate is not canonical");
    }
    Ok(field)
}

fn parse_g1(bytes: &[u8]) -> Result<G1Affine, &'static str> {
    if bytes.len() != 64 {
        return Err("invalid G1 encoding length");
    }
    Ok(G1Affine::new_unchecked(
        parse_fq(&bytes[..32])?,
        parse_fq(&bytes[32..])?,
    ))
}

fn parse_g2(bytes: &[u8]) -> Result<G2Affine, &'static str> {
    if bytes.len() != 128 {
        return Err("invalid G2 encoding length");
    }
    let x = ark_bn254::Fq2::new(parse_fq(&bytes[32..64])?, parse_fq(&bytes[..32])?);
    let y = ark_bn254::Fq2::new(parse_fq(&bytes[96..])?, parse_fq(&bytes[64..96])?);
    Ok(G2Affine::new_unchecked(x, y))
}
