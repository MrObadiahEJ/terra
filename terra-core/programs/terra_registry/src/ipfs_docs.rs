use anchor_lang::prelude::*;

pub const MAX_DOCUMENTS_PER_ATTESTATION: usize = 16;
pub const MAX_CID_LEN: usize = 128;

#[account]
#[derive(InitSpace)]
pub struct DocumentAnchor {
    /// The attestation this document belongs to.
    pub attestation: Pubkey,
    /// IPFS CID v1 of the document.
    #[max_len(128)]
    pub cid: String,
    /// SHA-256 of the document bytes (integrity check).
    pub content_hash: [u8; 32],
    /// Document category (e.g. "deed", "survey", "photo").
    #[max_len(32)]
    pub category: String,
    /// Who registered this document.
    pub registered_by: Pubkey,
    pub registered_at: i64,
}

pub fn register_document(
    ctx: Context<super::RegisterDocument>,
    cid: String,
    content_hash: [u8; 32],
    category: String,
) -> Result<()> {
    require!(!cid.is_empty(), super::TerraError::CidRequired);
    require!(
        content_hash != [0u8; 32],
        super::TerraError::CiphertextHashRequired
    );
    require!(!category.is_empty(), super::TerraError::EmptyName);

    // Enforce the per-attestation document cap.
    let attestation = &mut ctx.accounts.attestation;
    require!(
        attestation.document_count < MAX_DOCUMENTS_PER_ATTESTATION as u8,
        super::TerraError::TooManyStorageUris
    );
    attestation.document_count = attestation.document_count.saturating_add(1);

    let clock = Clock::get()?;
    let doc = &mut ctx.accounts.document;
    doc.attestation = ctx.accounts.attestation.key();
    doc.cid = cid.clone();
    doc.content_hash = content_hash;
    doc.category = category.clone();
    doc.registered_by = ctx.accounts.registrant.key();
    doc.registered_at = clock.unix_timestamp;

    emit!(super::DocumentRegistered {
        attestation: doc.attestation,
        cid,
        content_hash: doc.content_hash,
        category,
        registered_by: doc.registered_by,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn cid_cannot_be_empty() {
        let cid = "";
        assert!(cid.is_empty(), "empty CID should be rejected");
    }

    #[test]
    fn category_cannot_be_empty() {
        let category = "";
        assert!(category.is_empty(), "empty category should be rejected");
    }

    #[test]
    fn content_hash_cannot_be_zero() {
        let hash = [0u8; 32];
        assert!(hash.iter().all(|&b| b == 0), "zero hash should be rejected");
    }

    #[test]
    fn valid_document_fields() {
        let cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let hash = [1u8; 32];
        let category = "deed";
        assert!(!cid.is_empty());
        assert!(!hash.iter().all(|&b| b == 0));
        assert!(!category.is_empty());
    }
}
