//! Content-addressed, publisher-signed World Representation Manifests
//! (Chapter 12 §12.2/§12.8, ADR-020).
//!
//! A manifest is an immutable, versioned artifact. Its identity is the
//! SHA-256 digest of the canonical payload bytes (`ManifestPayload::to_bytes`,
//! fixed schema field order and no insignificant whitespace); its authenticity
//! is an Ed25519 signature over that digest, domain-separated by
//! [`MANIFEST_SIGNATURE_DOMAIN`]. Retrieval can therefore be content-addressed:
//! the caller asks for `sha256:<digest>`, checks the canonical payload bytes,
//! and only then trusts the manifest.
//!
//! The signature covers the **canonical payload serialization**, not an
//! arbitrary received encoding. Parsing re-serializes the payload to those
//! canonical bytes before hashing and verifying, so formatting differences are
//! tolerated while any semantic change is rejected. This avoids a
//! cross-language JSON canonicalization dependency while preserving the rule
//! that changed content can never keep the same address or signature.
//!
//! Manifest objects reference component versions, a content digest for the
//! component package bytes, a data-plane storage reference, bounds, available
//! LOD levels, and a validation state. They do not embed geometry.

use crate::spatial_index::{GeoBounds, QuadCell, SpatialIndexError};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;
use std::str::FromStr;

/// Schema tag carried by every signed manifest payload.
pub const MANIFEST_SCHEMA: &str = "terra.manifest/v1";

/// Domain separator signed ahead of the payload content address.
pub const MANIFEST_SIGNATURE_DOMAIN: &[u8] = b"terra.manifest.signature.v1\0";

/// Prefix used in content addresses.
pub const CONTENT_ADDRESS_PREFIX: &str = "sha256:";

/// Highest LOD level the runtime understands (Chapter 13: LOD 0–4).
pub const MAX_LOD: u8 = 4;

/// A SHA-256 content address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentAddress([u8; 32]);

impl ContentAddress {
    /// Hash arbitrary bytes with SHA-256.
    pub fn of(bytes: impl AsRef<[u8]>) -> Self {
        let digest = Sha256::digest(bytes.as_ref());
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Self(out)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Verify that `bytes` hash to this address.
    pub fn verify_bytes(&self, bytes: impl AsRef<[u8]>) -> bool {
        Self::of(bytes) == *self
    }
}

impl fmt::Display for ContentAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(CONTENT_ADDRESS_PREFIX)?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for ContentAddress {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s
            .strip_prefix(CONTENT_ADDRESS_PREFIX)
            .ok_or_else(|| ManifestError::InvalidContentAddress(s.to_string()))?;
        let bytes = decode_hex_exact::<32>(hex).map_err(ManifestError::InvalidContentAddress)?;
        Ok(Self(bytes))
    }
}

impl Serialize for ContentAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ContentAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// Validation state carried by a manifest object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState {
    Pending,
    TechnicallyValid,
    IndependentlyValid,
    Rejected,
}

/// One spatial object referenced by a manifest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManifestObject {
    pub object_id: String,
    pub component_version_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly_version_id: Option<String>,
    /// Digest of the component package bytes served by the data plane.
    pub content: ContentAddress,
    /// Data-plane retrieval reference (URL, provider key, …).
    pub storage_reference: String,
    pub bounds: GeoBounds,
    #[serde(default)]
    pub available_lods: Vec<u8>,
    pub validation_state: ValidationState,
}

/// The signed portion of a World Representation Manifest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManifestPayload {
    pub schema: String,
    pub region_id: String,
    pub region_name: String,
    /// Discovery partition for this manifest (see [`crate::spatial_index`]).
    pub region_cell: QuadCell,
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_manifest: Option<ContentAddress>,
    #[serde(default)]
    pub neighbor_manifests: Vec<ContentAddress>,
    pub objects: Vec<ManifestObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at_unix: i64,
}

impl ManifestPayload {
    pub fn new(
        region_id: impl Into<String>,
        region_name: impl Into<String>,
        region_cell: QuadCell,
        version: u32,
        objects: Vec<ManifestObject>,
        created_at_unix: i64,
    ) -> Self {
        Self {
            schema: MANIFEST_SCHEMA.to_string(),
            region_id: region_id.into(),
            region_name: region_name.into(),
            region_cell,
            version,
            parent_manifest: None,
            neighbor_manifests: Vec::new(),
            objects,
            description: None,
            created_at_unix,
        }
    }

    /// Validate the payload's semantic invariants before signing or trusting.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema != MANIFEST_SCHEMA {
            return Err(ManifestError::UnsupportedSchema(self.schema.clone()));
        }
        if self.region_id.trim().is_empty() {
            return Err(ManifestError::EmptyRegion);
        }
        if self.region_name.trim().is_empty() {
            return Err(ManifestError::EmptyRegionName);
        }
        if self.version == 0 {
            return Err(ManifestError::InvalidVersion);
        }

        let region_bounds = self.region_cell.bounds();
        let mut object_ids = BTreeSet::new();
        for object in &self.objects {
            if object.object_id.trim().is_empty() {
                return Err(ManifestError::InvalidObject(
                    "<empty>".to_string(),
                    "object_id must not be empty".to_string(),
                ));
            }
            if object.component_version_id.trim().is_empty() {
                return Err(ManifestError::InvalidObject(
                    object.object_id.clone(),
                    "component_version_id must not be empty".to_string(),
                ));
            }
            if object.storage_reference.trim().is_empty() {
                return Err(ManifestError::InvalidObject(
                    object.object_id.clone(),
                    "storage_reference must not be empty".to_string(),
                ));
            }
            object.bounds.validate().map_err(|e| {
                ManifestError::InvalidObject(object.object_id.clone(), e.to_string())
            })?;
            if !region_bounds.encloses(&object.bounds) {
                return Err(ManifestError::InvalidObject(
                    object.object_id.clone(),
                    format!(
                        "bounds [{}, {}] .. [{}, {}] fall outside region cell {}",
                        object.bounds.min_lon,
                        object.bounds.min_lat,
                        object.bounds.max_lon,
                        object.bounds.max_lat,
                        self.region_cell
                    ),
                ));
            }
            let mut lods = BTreeSet::new();
            for lod in &object.available_lods {
                if *lod > MAX_LOD {
                    return Err(ManifestError::InvalidObject(
                        object.object_id.clone(),
                        format!("LOD {lod} exceeds maximum {MAX_LOD}"),
                    ));
                }
                if !lods.insert(*lod) {
                    return Err(ManifestError::InvalidObject(
                        object.object_id.clone(),
                        format!("duplicate LOD {lod}"),
                    ));
                }
            }
            if !object_ids.insert(&object.object_id) {
                return Err(ManifestError::DuplicateObject(object.object_id.clone()));
            }
        }

        let mut neighbors = BTreeSet::new();
        for neighbor in &self.neighbor_manifests {
            if !neighbors.insert(neighbor) {
                return Err(ManifestError::DuplicateNeighbor(neighbor.to_string()));
            }
        }

        Ok(())
    }

    /// Validate and serialize the exact payload bytes that are signed and
    /// content-addressed.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ManifestError> {
        self.validate()?;
        Ok(serde_json::to_vec(self)?)
    }

    /// Content address of the exact serialized payload bytes.
    pub fn content_address(&self) -> Result<ContentAddress, ManifestError> {
        Ok(ContentAddress::of(self.to_bytes()?))
    }
}

/// A payload plus its publisher key and Ed25519 signature.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignedManifest {
    pub payload: ManifestPayload,
    /// Hex-encoded 32-byte Ed25519 public key.
    pub publisher: String,
    /// Hex-encoded 64-byte Ed25519 signature.
    pub signature: String,
}

impl SignedManifest {
    /// Sign a validated payload with an Ed25519 key.
    pub fn sign(payload: ManifestPayload, key: &SigningKey) -> Result<Self, ManifestError> {
        let address = payload.content_address()?;
        let signature = key.sign(&signing_message(&address));
        Ok(Self {
            payload,
            publisher: encode_hex(key.verifying_key().as_bytes()),
            signature: encode_hex(&signature.to_bytes()),
        })
    }

    /// Recompute the payload content address.
    pub fn content_address(&self) -> Result<ContentAddress, ManifestError> {
        self.payload.content_address()
    }

    /// Validate the payload, then verify the signature and return the content
    /// address on success.
    pub fn verify(&self) -> Result<ContentAddress, ManifestError> {
        let address = self.payload.content_address()?;
        let key_bytes =
            decode_hex_exact::<32>(&self.publisher).map_err(ManifestError::InvalidPublisher)?;
        let key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| ManifestError::InvalidPublisher(e.to_string()))?;
        let signature_bytes = decode_hex_exact::<64>(&self.signature)
            .map_err(ManifestError::InvalidSignatureEncoding)?;
        let signature = Signature::from_bytes(&signature_bytes);
        key.verify(&signing_message(&address), &signature)
            .map_err(|_| ManifestError::SignatureMismatch)?;
        Ok(address)
    }

    /// Serialize after verification.
    pub fn to_json(&self) -> Result<String, ManifestError> {
        self.verify()?;
        Ok(serde_json::to_string(self)?)
    }

    /// Parse and verify in one step. Rejects tampered or malformed manifests.
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_str(json)?;
        manifest.verify()?;
        Ok(manifest)
    }
}

/// Manifest-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("unsupported manifest schema `{0}` (expected `{MANIFEST_SCHEMA}`)")]
    UnsupportedSchema(String),
    #[error("manifest region_id must not be empty")]
    EmptyRegion,
    #[error("manifest region_name must not be empty")]
    EmptyRegionName,
    #[error("manifest version must be >= 1")]
    InvalidVersion,
    #[error("manifest object `{0}` is invalid: {1}")]
    InvalidObject(String, String),
    #[error("duplicate object id `{0}` in manifest")]
    DuplicateObject(String),
    #[error("duplicate neighbor manifest `{0}`")]
    DuplicateNeighbor(String),
    #[error("invalid content address `{0}`")]
    InvalidContentAddress(String),
    #[error("invalid publisher key: {0}")]
    InvalidPublisher(String),
    #[error("invalid signature encoding: {0}")]
    InvalidSignatureEncoding(String),
    #[error("manifest signature does not match payload and publisher")]
    SignatureMismatch,
    #[error("invalid spatial index value: {0}")]
    Spatial(#[from] SpatialIndexError),
    #[error("manifest JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

fn signing_message(address: &ContentAddress) -> Vec<u8> {
    let mut message = Vec::with_capacity(MANIFEST_SIGNATURE_DOMAIN.len() + 32);
    message.extend_from_slice(MANIFEST_SIGNATURE_DOMAIN);
    message.extend_from_slice(address.as_bytes());
    message
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn decode_hex_exact<const N: usize>(hex: &str) -> Result<[u8; N], String> {
    if hex.len() != N * 2 {
        return Err(format!(
            "expected {} hex characters, got {}",
            N * 2,
            hex.len()
        ));
    }
    let mut out = [0u8; N];
    for (i, byte) in out.iter_mut().enumerate() {
        let start = i * 2;
        *byte = u8::from_str_radix(&hex[start..start + 2], 16)
            .map_err(|_| format!("invalid hex at byte {i}"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell() -> QuadCell {
        QuadCell::from_point(11.5, 3.85, 8).unwrap()
    }

    fn object(object_id: &str) -> ManifestObject {
        ManifestObject {
            object_id: object_id.to_string(),
            component_version_id: format!("{object_id}-v1"),
            assembly_version_id: None,
            content: ContentAddress::of(format!("package:{object_id}")),
            storage_reference: format!("ipfs://{object_id}"),
            bounds: GeoBounds::new(11.49, 3.84, 11.51, 3.86).unwrap(),
            available_lods: vec![0, 1, 2],
            validation_state: ValidationState::TechnicallyValid,
        }
    }

    fn payload() -> ManifestPayload {
        ManifestPayload::new(
            "cm-centre",
            "Centre, Cameroon",
            cell(),
            1,
            vec![object("parcel-1"), object("parcel-2")],
            1_700_000_000,
        )
    }

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    #[test]
    fn content_address_roundtrip_and_verification() {
        let address = ContentAddress::of(b"hello world");
        assert!(address.verify_bytes(b"hello world"));
        assert!(!address.verify_bytes(b"hello world!"));
        let text = address.to_string();
        assert!(text.starts_with(CONTENT_ADDRESS_PREFIX));
        assert_eq!(text.parse::<ContentAddress>().unwrap(), address);
        assert!("sha256:xyz".parse::<ContentAddress>().is_err());
    }

    #[test]
    fn signed_manifest_roundtrip() {
        let manifest = SignedManifest::sign(payload(), &key()).unwrap();
        let address = manifest.verify().unwrap();
        assert_eq!(address, manifest.content_address().unwrap());

        let json = manifest.to_json().unwrap();
        let parsed = SignedManifest::from_json(&json).unwrap();
        assert_eq!(parsed, manifest);
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let mut manifest = SignedManifest::sign(payload(), &key()).unwrap();
        manifest.payload.version = 2;
        assert!(matches!(
            manifest.verify(),
            Err(ManifestError::SignatureMismatch)
        ));
    }

    #[test]
    fn wrong_publisher_is_rejected() {
        let mut manifest = SignedManifest::sign(payload(), &key()).unwrap();
        manifest.publisher = encode_hex(
            &SigningKey::from_bytes(&[9u8; 32])
                .verifying_key()
                .to_bytes(),
        );
        assert!(matches!(
            manifest.verify(),
            Err(ManifestError::SignatureMismatch)
        ));
    }

    #[test]
    fn payload_validation_rejects_bad_objects() {
        let mut p = payload();
        p.version = 0;
        assert!(matches!(p.validate(), Err(ManifestError::InvalidVersion)));

        let mut p = payload();
        p.objects.push(object("parcel-1"));
        assert!(matches!(
            p.validate(),
            Err(ManifestError::DuplicateObject(id)) if id == "parcel-1"
        ));

        let mut p = payload();
        p.objects[0].available_lods = vec![0, 5];
        assert!(p.validate().is_err());

        let mut p = payload();
        p.objects[0].bounds = GeoBounds::new(-10.0, -10.0, 10.0, 10.0).unwrap();
        assert!(matches!(
            p.validate(),
            Err(ManifestError::InvalidObject(_, _))
        ));
    }

    #[test]
    fn payload_schema_and_region_are_checked() {
        let mut p = payload();
        p.schema = "terra.manifest/v0".into();
        assert!(matches!(
            p.validate(),
            Err(ManifestError::UnsupportedSchema(_))
        ));

        let mut p = payload();
        p.region_id = "  ".into();
        assert!(matches!(p.validate(), Err(ManifestError::EmptyRegion)));

        let mut p = payload();
        p.region_name = String::new();
        assert!(matches!(p.validate(), Err(ManifestError::EmptyRegionName)));
    }

    #[test]
    fn duplicate_neighbors_are_rejected() {
        let mut p = payload();
        let neighbor = ContentAddress::of(b"neighbor");
        p.neighbor_manifests = vec![neighbor, neighbor];
        assert!(matches!(
            p.validate(),
            Err(ManifestError::DuplicateNeighbor(_))
        ));
    }

    #[test]
    fn json_roundtrip_preserves_content_address() {
        let p = payload();
        let address = p.content_address().unwrap();
        let json = serde_json::to_string(&p).unwrap();
        let parsed: ManifestPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
        assert_eq!(parsed.content_address().unwrap(), address);
    }
}
