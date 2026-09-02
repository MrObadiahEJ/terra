/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/terra_registry.json`.
 */
export type TerraRegistry = 
{
  "address": "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage",
  "metadata": {
    "name": "terra_registry",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "add_validator_to_registry",
      "discriminator": [
        117,
        200,
        150,
        251,
        179,
        39,
        244,
        40
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "admin_signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "endorsement",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  108,
                  105,
                  100,
                  97,
                  116,
                  111,
                  114,
                  95,
                  101,
                  110,
                  100,
                  111,
                  114,
                  115,
                  101,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "registry"
              },
              {
                "kind": "arg",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator"
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "validator",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "attach_parcel",
      "docs": [
        "Attach a parcel to an identity (the person behind its owner wallet).",
        "Only the parcel's owner may do this, and only for an identity whose",
        "owner wallet matches."
      ],
      "discriminator": [
        99,
        215,
        88,
        205,
        231,
        118,
        0,
        81
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "identity",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  101,
                  110,
                  116,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "identity.identity_hash",
                "account": "Identity"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "attest",
      "docs": [
        "Register an attestation that binds heavy off-chain data to this parcel",
        "and records the set of validator wallets required to validate it.",
        "",
        "`validators` holds the public keys of the (possibly several) parties",
        "who must sign off on the transaction; `required` is how many signatures",
        "are needed. The signer must be the parcel owner or a registered",
        "registrar. Per-validator Ed25519 signatures live off-chain but are",
        "verified against this on-chain identity set and `content_hash`."
      ],
      "discriminator": [
        83,
        148,
        120,
        119,
        144,
        139,
        117,
        160
      ],
      "accounts": [
        {
          "name": "parcel",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "attestation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "arg",
                "path": "specifier"
              }
            ]
          }
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "specifier",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "content_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "required",
          "type": "u8"
        },
        {
          "name": "validators",
          "type": {
            "array": [
              "pubkey",
              8
            ]
          }
        }
      ]
    },
    {
      "name": "authorize_vault_access",
      "discriminator": [
        241,
        177,
        105,
        203,
        97,
        122,
        137,
        9
      ],
      "accounts": [
        {
          "name": "vault_record",
          "writable": true
        },
        {
          "name": "subject"
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "purpose",
          "type": "string"
        },
        {
          "name": "expiry",
          "type": "i64"
        },
        {
          "name": "off_chain_nonce",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "bind_identity",
      "docs": [
        "Bind a person (identified by a hashed credential) to a wallet the person",
        "holds. `recovery` is a second wallet the person controls, for recovering",
        "the identity if the main key is lost. The signer becomes `owner`.",
        "",
        "This is the root of the resolvable \"who owns this\" link: every on-chain",
        "actor is ultimately a wallet, and this account binds that wallet to a",
        "human without ever publishing the credential itself."
      ],
      "discriminator": [
        233,
        223,
        188,
        85,
        140,
        1,
        204,
        196
      ],
      "accounts": [
        {
          "name": "identity",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  101,
                  110,
                  116,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "arg",
                "path": "identity_hash"
              }
            ]
          }
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "identity_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "recovery",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "cancel_shard_rotation",
      "discriminator": [
        41,
        160,
        228,
        229,
        31,
        131,
        109,
        12
      ],
      "accounts": [
        {
          "name": "rotation",
          "writable": true
        },
        {
          "name": "vault_record"
        },
        {
          "name": "canceller",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "cancel_succession",
      "docs": [
        "Cancel an in-flight succession. Only the current `owner` (or `recovery`",
        "for a recovery passation) may cancel, and only before it is effective."
      ],
      "discriminator": [
        206,
        172,
        126,
        132,
        252,
        50,
        143,
        214
      ],
      "accounts": [
        {
          "name": "identity",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  101,
                  110,
                  116,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "identity.identity_hash",
                "account": "Identity"
              }
            ]
          }
        },
        {
          "name": "succession",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  117,
                  99,
                  99,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "succession.identity",
                "account": "Succession"
              },
              {
                "kind": "account",
                "path": "succession.successor",
                "account": "Succession"
              }
            ]
          }
        },
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "claim_succession",
      "docs": [
        "Claim a passation once BOTH the grace period has elapsed AND the required",
        "number of validators have endorsed it. The `successor` becomes the",
        "identity's new owner. Any parcels the identity owned that are supplied",
        "via `remaining_accounts` are re-pointed to the successor."
      ],
      "discriminator": [
        90,
        253,
        80,
        136,
        147,
        71,
        124,
        7
      ],
      "accounts": [
        {
          "name": "identity",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  101,
                  110,
                  116,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "identity.identity_hash",
                "account": "Identity"
              }
            ]
          }
        },
        {
          "name": "succession",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  117,
                  99,
                  99,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "succession.identity",
                "account": "Succession"
              },
              {
                "kind": "account",
                "path": "succession.successor",
                "account": "Succession"
              }
            ]
          }
        },
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "create_registry",
      "discriminator": [
        210,
        219,
        233,
        49,
        251,
        19,
        135,
        13
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121,
                  95,
                  114,
                  101,
                  103,
                  105,
                  115,
                  116,
                  114,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "create_vault",
      "discriminator": [
        29,
        237,
        247,
        208,
        193,
        82,
        54,
        135
      ],
      "accounts": [
        {
          "name": "vault_record",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116,
                  95,
                  114,
                  101,
                  99,
                  111,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "subject"
              }
            ]
          }
        },
        {
          "name": "subject"
        },
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "ciphertext_cid",
          "type": "string"
        },
        {
          "name": "ciphertext_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "algorithm_id",
          "type": "u8"
        },
        {
          "name": "storage_uris",
          "type": {
            "vec": "string"
          }
        },
        {
          "name": "shard_holders",
          "type": {
            "vec": "pubkey"
          }
        },
        {
          "name": "threshold",
          "type": "u8"
        }
      ]
    },
    {
      "name": "endorse_shard_rotation",
      "discriminator": [
        132,
        217,
        137,
        226,
        97,
        88,
        106,
        72
      ],
      "accounts": [
        {
          "name": "rotation",
          "writable": true
        },
        {
          "name": "vault_record"
        },
        {
          "name": "subject"
        },
        {
          "name": "validator",
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "endorse_succession",
      "docs": [
        "Record one validator's endorsement of a pending succession. The signing",
        "validator must be in the succession's declared validator set; this bumps",
        "`validations_count`. Each endorsement is an Ed25519 signature because the",
        "validator signs this transaction with their wallet. Only meaningful",
        "before the succession becomes effective (validations are then moot)."
      ],
      "discriminator": [
        70,
        125,
        62,
        184,
        108,
        245,
        74,
        100
      ],
      "accounts": [
        {
          "name": "identity",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  101,
                  110,
                  116,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "identity.identity_hash",
                "account": "Identity"
              }
            ]
          }
        },
        {
          "name": "succession",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  117,
                  99,
                  99,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "succession.identity",
                "account": "Succession"
              },
              {
                "kind": "account",
                "path": "succession.successor",
                "account": "Succession"
              }
            ]
          }
        },
        {
          "name": "validator",
          "docs": [
            "A declared local validator endorsing the passation (signs this tx)."
          ],
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "endorse_validator_add",
      "discriminator": [
        42,
        252,
        185,
        227,
        148,
        64,
        141,
        174
      ],
      "accounts": [
        {
          "name": "endorsement",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  108,
                  105,
                  100,
                  97,
                  116,
                  111,
                  114,
                  95,
                  101,
                  110,
                  100,
                  111,
                  114,
                  115,
                  101,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "registry"
              },
              {
                "kind": "account",
                "path": "endorsement.proposed",
                "account": "ValidatorEndorsement"
              }
            ]
          }
        },
        {
          "name": "registry"
        },
        {
          "name": "endorser",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "execute_shard_rotation",
      "discriminator": [
        241,
        227,
        238,
        115,
        108,
        210,
        193,
        101
      ],
      "accounts": [
        {
          "name": "rotation",
          "writable": true
        },
        {
          "name": "vault_record",
          "writable": true
        },
        {
          "name": "initiator",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "flip_to_consensus",
      "discriminator": [
        123,
        232,
        56,
        243,
        82,
        102,
        120,
        239
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "admin_signer",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "grant_right",
      "docs": [
        "Grant a right on a parcel to `holder`. Owner-only.",
        "",
        "`nonce` must equal the parcel's current `rights_count`, which is",
        "incremented so every right gets a unique PDA."
      ],
      "discriminator": [
        147,
        166,
        175,
        167,
        132,
        161,
        76,
        232
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "rights",
          "writable": true
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "nonce",
          "type": "u8"
        },
        {
          "name": "rights_kind",
          "type": "u8"
        },
        {
          "name": "holder",
          "type": "pubkey"
        },
        {
          "name": "expires_at",
          "type": "i64"
        },
        {
          "name": "notes",
          "type": "string"
        }
      ]
    },
    {
      "name": "initiate_shard_rotation",
      "discriminator": [
        48,
        48,
        204,
        149,
        150,
        34,
        235,
        97
      ],
      "accounts": [
        {
          "name": "rotation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116,
                  95,
                  115,
                  104,
                  97,
                  114,
                  100,
                  95,
                  114,
                  111,
                  116,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "vault_record"
              },
              {
                "kind": "arg",
                "path": "new_ciphertext_hash"
              }
            ]
          }
        },
        {
          "name": "vault_record",
          "writable": true
        },
        {
          "name": "initiator",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "new_ciphertext_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "new_shard_holders",
          "type": {
            "vec": "pubkey"
          }
        },
        {
          "name": "new_threshold",
          "type": "u8"
        }
      ]
    },
    {
      "name": "judicial_forfeiture",
      "docs": [
        "Force-transfer a parcel's ownership away from a non-compliant owner, per",
        "a court order. This is deliberately heavier than a normal transfer:",
        "at least `MIN_FORFEIT_VALIDATORS` (2) of the declared validators must",
        "sign this transaction themselves, and the order is bound to a",
        "`case_hash` (e.g. SHA-256 of the court order document) for auditability.",
        "",
        "This is how validators collectively inform the chain that land no longer",
        "belongs to someone who refuses to release it \u2014 e.g. repossession by a",
        "government, or a court ruling that title passed to another person."
      ],
      "discriminator": [
        34,
        185,
        214,
        40,
        233,
        253,
        255,
        20
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "authority",
          "docs": [
            "Relaying authority (court clerk / govt channel). Must NOT be the owner."
          ],
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "case_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "new_owner",
          "type": "pubkey"
        },
        {
          "name": "threshold",
          "type": "u8"
        },
        {
          "name": "validators",
          "type": {
            "array": [
              "pubkey",
              8
            ]
          }
        }
      ]
    },
    {
      "name": "ping_shard",
      "discriminator": [
        148,
        232,
        132,
        244,
        167,
        20,
        100,
        215
      ],
      "accounts": [
        {
          "name": "vault_record",
          "writable": true
        },
        {
          "name": "validator",
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "register_document",
      "discriminator": [
        108,
        34,
        153,
        39,
        82,
        41,
        133,
        73
      ],
      "accounts": [
        {
          "name": "document",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  111,
                  99,
                  117,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "attestation"
              },
              {
                "kind": "arg",
                "path": "cid"
              }
            ]
          }
        },
        {
          "name": "attestation",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "account",
                "path": "attestation.specifier",
                "account": "Attestation"
              }
            ]
          }
        },
        {
          "name": "parcel",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "registrant",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "cid",
          "type": "string"
        },
        {
          "name": "content_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "category",
          "type": "string"
        }
      ]
    },
    {
      "name": "register_parcel",
      "docs": [
        "Register a new parcel on-chain. The signer becomes its owner.",
        "",
        "`id` is a caller-provided unique 32-byte identifier (e.g. a SHA-256 of",
        "the parcel geometry). It is also the PDA seed, so it can never change."
      ],
      "discriminator": [
        170,
        232,
        221,
        44,
        109,
        149,
        104,
        207
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "arg",
                "path": "id"
              }
            ]
          }
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "id",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "geometry_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "remove_validator_from_registry",
      "discriminator": [
        62,
        197,
        103,
        150,
        26,
        158,
        116,
        235
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "admin_signer",
          "signer": true
        },
        {
          "name": "endorsement",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  108,
                  105,
                  100,
                  97,
                  116,
                  111,
                  114,
                  95,
                  101,
                  110,
                  100,
                  111,
                  114,
                  115,
                  101,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "registry"
              },
              {
                "kind": "arg",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator"
        }
      ],
      "args": [
        {
          "name": "validator",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "request_succession",
      "docs": [
        "Request a wallet passation (succession, recovery, or deliberate control",
        "transfer). A Succession account is created and becomes effective only",
        "after the grace period \u2014 within which the original owner can cancel.",
        "",
        "Authorized by the current `owner` for kind TRANSFER, or by the `owner`",
        "OR the `recovery` wallet for kind RECOVERY/SUCCESSOR.",
        "",
        "`grace_secs` lets the requester choose the window (0 => default 30d),",
        "clamped to [MIN, MAX]. `required_validations` is the number of declared",
        "local validators that must endorse the passation before it can be",
        "claimed (>= 1) \u2014 so a stolen wallet can't seize land alone.",
        "`validators` declares the local-authority testifiers for this passation."
      ],
      "discriminator": [
        239,
        203,
        74,
        151,
        24,
        159,
        159,
        84
      ],
      "accounts": [
        {
          "name": "identity",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  100,
                  101,
                  110,
                  116,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "identity.identity_hash",
                "account": "Identity"
              }
            ]
          }
        },
        {
          "name": "succession",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  117,
                  99,
                  99,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "identity"
              },
              {
                "kind": "arg",
                "path": "successor"
              }
            ]
          }
        },
        {
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "successor",
          "type": "pubkey"
        },
        {
          "name": "kind",
          "type": "u8"
        },
        {
          "name": "grace_secs",
          "type": "i64"
        },
        {
          "name": "required_validations",
          "type": "u8"
        },
        {
          "name": "validators",
          "type": {
            "array": [
              "pubkey",
              8
            ]
          }
        }
      ]
    },
    {
      "name": "revoke_right",
      "docs": [
        "Revoke a previously granted right. The parcel owner or the original",
        "granter may revoke. The account is closed and its lamports returned."
      ],
      "discriminator": [
        209,
        129,
        92,
        98,
        174,
        82,
        72,
        77
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "rights",
          "writable": true
        },
        {
          "name": "owner",
          "writable": true,
          "signer": true
        },
        {
          "name": "system_program",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "_nonce",
          "type": "u8"
        }
      ]
    },
    {
      "name": "rotate_validators",
      "docs": [
        "Replace the validator set on an attestation (the fix for dead/leaving",
        "validators). Only the parcel owner may rotate. Bumps `version` so a",
        "reconstituted set is provably newer, and resets `required`/`count`."
      ],
      "discriminator": [
        98,
        183,
        54,
        7,
        187,
        27,
        218,
        242
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "attestation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "account",
                "path": "attestation.specifier",
                "account": "Attestation"
              }
            ]
          }
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "new_required",
          "type": "u8"
        },
        {
          "name": "new_validators",
          "type": {
            "array": [
              "pubkey",
              8
            ]
          }
        }
      ]
    },
    {
      "name": "transfer_parcel",
      "docs": [
        "Transfer ownership of a parcel. Only the current owner can sign."
      ],
      "discriminator": [
        214,
        120,
        155,
        187,
        215,
        201,
        59,
        129
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "new_owner"
        }
      ],
      "args": []
    },
    {
      "name": "update_infrastructure",
      "docs": [
        "Set the parcel's infrastructure flag bitmask together with the canonical",
        "access digest produced by the off-chain validation engine. Owner-only.",
        "",
        "`access_hash` must be non-zero and match the digests the off-chain",
        "engine derives for these flags on the parcel geometry."
      ],
      "discriminator": [
        166,
        23,
        147,
        198,
        105,
        63,
        108,
        237
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "flags",
          "type": "u16"
        },
        {
          "name": "access_hash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "update_status",
      "docs": [
        "Update a parcel's status (e.g. for-sale). Owner-only."
      ],
      "discriminator": [
        147,
        215,
        74,
        174,
        55,
        191,
        42,
        0
      ],
      "accounts": [
        {
          "name": "parcel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  97,
                  114,
                  99,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "parcel.id",
                "account": "Parcel"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "status",
          "type": "u8"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "Attestation",
      "discriminator": [
        152,
        125,
        183,
        86,
        36,
        146,
        121,
        73
      ]
    },
    {
      "name": "Identity",
      "discriminator": [
        58,
        132,
        5,
        12,
        176,
        164,
        85,
        112
      ]
    },
    {
      "name": "Parcel",
      "discriminator": [
        149,
        167,
        245,
        67,
        209,
        244,
        214,
        75
      ]
    },
    {
      "name": "Rights",
      "discriminator": [
        79,
        187,
        86,
        85,
        243,
        213,
        103,
        234
      ]
    },
    {
      "name": "Succession",
      "discriminator": [
        51,
        87,
        221,
        243,
        105,
        19,
        68,
        109
      ]
    },
    {
      "name": "VaultRecord",
      "discriminator": [
        47,
        1,
        218,
        116,
        82,
        70,
        124,
        119
      ]
    },
    {
      "name": "VaultShardRotation",
      "discriminator": [
        33,
        187,
        116,
        141,
        215,
        66,
        44,
        139
      ]
    },
    {
      "name": "AuthorityRegistry",
      "discriminator": [
        239,
        214,
        161,
        141,
        212,
        86,
        122,
        109
      ]
    },
    {
      "name": "DocumentAnchor",
      "discriminator": [
        60,
        3,
        60,
        133,
        60,
        201,
        124,
        184
      ]
    },
    {
      "name": "ValidatorEndorsement",
      "discriminator": [
        140,
        204,
        234,
        117,
        105,
        8,
        203,
        25
      ]
    }
  ],
  "events": [
    {
      "name": "Attested",
      "discriminator": [
        184,
        102,
        113,
        199,
        220,
        197,
        96,
        50
      ]
    },
    {
      "name": "IdentityBound",
      "discriminator": [
        183,
        169,
        144,
        11,
        110,
        67,
        103,
        46
      ]
    },
    {
      "name": "InfrastructureUpdated",
      "discriminator": [
        198,
        12,
        158,
        169,
        233,
        83,
        181,
        172
      ]
    },
    {
      "name": "ParcelAttached",
      "discriminator": [
        92,
        181,
        155,
        58,
        230,
        193,
        129,
        6
      ]
    },
    {
      "name": "ParcelForfeited",
      "discriminator": [
        169,
        221,
        21,
        91,
        124,
        141,
        210,
        97
      ]
    },
    {
      "name": "ParcelRegistered",
      "discriminator": [
        135,
        44,
        33,
        59,
        6,
        1,
        21,
        83
      ]
    },
    {
      "name": "ParcelTransferred",
      "discriminator": [
        242,
        85,
        211,
        205,
        7,
        84,
        243,
        129
      ]
    },
    {
      "name": "RightGranted",
      "discriminator": [
        251,
        86,
        219,
        230,
        109,
        252,
        94,
        16
      ]
    },
    {
      "name": "RightRevoked",
      "discriminator": [
        67,
        219,
        188,
        196,
        202,
        34,
        65,
        52
      ]
    },
    {
      "name": "RotationEndorsed",
      "discriminator": [
        238,
        193,
        191,
        93,
        217,
        213,
        91,
        230
      ]
    },
    {
      "name": "ShardPinged",
      "discriminator": [
        114,
        228,
        60,
        52,
        97,
        79,
        88,
        55
      ]
    },
    {
      "name": "ShardRotationCancelled",
      "discriminator": [
        148,
        213,
        25,
        213,
        152,
        212,
        90,
        31
      ]
    },
    {
      "name": "ShardRotationExecuted",
      "discriminator": [
        207,
        240,
        193,
        208,
        187,
        113,
        36,
        135
      ]
    },
    {
      "name": "ShardRotationInitiated",
      "discriminator": [
        193,
        160,
        222,
        157,
        254,
        127,
        137,
        223
      ]
    },
    {
      "name": "SuccessionCancelled",
      "discriminator": [
        67,
        67,
        101,
        243,
        100,
        8,
        158,
        53
      ]
    },
    {
      "name": "SuccessionClaimed",
      "discriminator": [
        27,
        211,
        3,
        154,
        93,
        191,
        66,
        212
      ]
    },
    {
      "name": "SuccessionEndorsed",
      "discriminator": [
        207,
        163,
        114,
        67,
        56,
        134,
        119,
        241
      ]
    },
    {
      "name": "SuccessionRequested",
      "discriminator": [
        212,
        119,
        41,
        85,
        179,
        61,
        206,
        98
      ]
    },
    {
      "name": "ValidatorsRotated",
      "discriminator": [
        80,
        217,
        37,
        28,
        47,
        73,
        79,
        88
      ]
    },
    {
      "name": "VaultAccessAuthorized",
      "discriminator": [
        76,
        112,
        168,
        177,
        253,
        40,
        233,
        150
      ]
    },
    {
      "name": "VaultCreated",
      "discriminator": [
        117,
        25,
        120,
        254,
        75,
        236,
        78,
        115
      ]
    },
    {
      "name": "ConsensusFlipped",
      "discriminator": [
        138,
        151,
        228,
        234,
        58,
        84,
        243,
        252
      ]
    },
    {
      "name": "DocumentRegistered",
      "discriminator": [
        39,
        98,
        72,
        173,
        200,
        16,
        169,
        166
      ]
    },
    {
      "name": "RegistryCreated",
      "discriminator": [
        155,
        150,
        75,
        69,
        222,
        185,
        234,
        132
      ]
    },
    {
      "name": "ValidatorAdded",
      "discriminator": [
        67,
        26,
        43,
        25,
        58,
        219,
        99,
        48
      ]
    },
    {
      "name": "ValidatorEndorsed",
      "discriminator": [
        117,
        229,
        82,
        229,
        85,
        201,
        73,
        81
      ]
    },
    {
      "name": "ValidatorRemoved",
      "discriminator": [
        133,
        140,
        80,
        83,
        7,
        209,
        70,
        130
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InvalidId",
      "msg": "Parcel id cannot be all zeros"
    },
    {
      "code": 6001,
      "name": "EmptyName",
      "msg": "Parcel name cannot be empty"
    },
    {
      "code": 6002,
      "name": "EmptyGeometryHash",
      "msg": "Geometry hash is required"
    },
    {
      "code": 6003,
      "name": "NotOwner",
      "msg": "Only the current owner can perform this action"
    },
    {
      "code": 6004,
      "name": "InvalidStatus",
      "msg": "Invalid parcel status"
    },
    {
      "code": 6005,
      "name": "InvalidRightKind",
      "msg": "Invalid right kind"
    },
    {
      "code": 6006,
      "name": "InvalidNonce",
      "msg": "Nonce does not match the parcel's rights_count"
    },
    {
      "code": 6007,
      "name": "RightsLimitExceeded",
      "msg": "Rights limit reached"
    },
    {
      "code": 6008,
      "name": "NotesTooLong",
      "msg": "Notes exceed the maximum length of 128"
    },
    {
      "code": 6009,
      "name": "InvalidExpiry",
      "msg": "Expiry must be in the future"
    },
    {
      "code": 6010,
      "name": "NotAuthorized",
      "msg": "Not authorized to perform this action"
    },
    {
      "code": 6011,
      "name": "InvalidInfrastructureFlags",
      "msg": "Invalid infrastructure flags"
    },
    {
      "code": 6012,
      "name": "EmptyAccessHash",
      "msg": "Access hash is required"
    },
    {
      "code": 6013,
      "name": "EmptySpecifier",
      "msg": "Attestation specifier is required"
    },
    {
      "code": 6014,
      "name": "EmptyContentHash",
      "msg": "Content hash is required"
    },
    {
      "code": 6015,
      "name": "NoValidators",
      "msg": "Attestation requires at least one validator"
    },
    {
      "code": 6016,
      "name": "InvalidThreshold",
      "msg": "Required threshold exceeds the number of validators"
    },
    {
      "code": 6017,
      "name": "EmptyIdentityHash",
      "msg": "Identity hash is required"
    },
    {
      "code": 6018,
      "name": "EmptyRecovery",
      "msg": "Recovery wallet is required"
    },
    {
      "code": 6019,
      "name": "IdentityMismatch",
      "msg": "Identity owner does not match the parcel owner"
    },
    {
      "code": 6020,
      "name": "EmptySuccessor",
      "msg": "Successor wallet is required"
    },
    {
      "code": 6021,
      "name": "InvalidSuccessionKind",
      "msg": "Invalid succession kind"
    },
    {
      "code": 6022,
      "name": "SuccessorIsOwner",
      "msg": "Successor must differ from the current owner"
    },
    {
      "code": 6023,
      "name": "SuccessionAlreadyEffective",
      "msg": "Succession has already become effective"
    },
    {
      "code": 6024,
      "name": "NotSuccessor",
      "msg": "Only the named successor may claim this succession"
    },
    {
      "code": 6025,
      "name": "SuccessionNotYetEffective",
      "msg": "Succession is not yet effective"
    },
    {
      "code": 6026,
      "name": "AttestationMismatch",
      "msg": "Attestation does not belong to this parcel"
    },
    {
      "code": 6027,
      "name": "InsufficientValidations",
      "msg": "Succession requires validator endorsements before it can be claimed"
    },
    {
      "code": 6028,
      "name": "NotValidator",
      "msg": "Signing wallet is not a declared validator for this succession"
    },
    {
      "code": 6029,
      "name": "ValidationLimitReached",
      "msg": "No more validators may endorse this succession (limit reached)"
    },
    {
      "code": 6030,
      "name": "EmptyCaseHash",
      "msg": "Court case hash is required"
    },
    {
      "code": 6031,
      "name": "EmptyNewOwner",
      "msg": "New forfeiture owner is required"
    },
    {
      "code": 6032,
      "name": "InsufficientValidatorSigners",
      "msg": "Not enough validator signers to forfeit this parcel"
    },
    {
      "code": 6033,
      "name": "OwnerCannotSelfForfeit",
      "msg": "The current owner cannot self-forfeit their own parcel"
    },
    {
      "code": 6034,
      "name": "ValidatorOwnsAsset",
      "msg": "A validator cannot be the owner of the asset being validated"
    },
    {
      "code": 6035,
      "name": "VaultAlreadyExists",
      "msg": "Vault already exists for this subject"
    },
    {
      "code": 6036,
      "name": "VaultNotFound",
      "msg": "Vault not found"
    },
    {
      "code": 6037,
      "name": "ThresholdExceedsHolders",
      "msg": "Threshold exceeds the number of shard holders"
    },
    {
      "code": 6038,
      "name": "NotShardHolder",
      "msg": "Signer is not a shard holder for this vault"
    },
    {
      "code": 6039,
      "name": "NotActiveValidator",
      "msg": "Signer is not an active validator in this vault"
    },
    {
      "code": 6040,
      "name": "CiphertextHashRequired",
      "msg": "Ciphertext hash cannot be all zeros"
    },
    {
      "code": 6041,
      "name": "CidRequired",
      "msg": "Ciphertext CID cannot be empty"
    },
    {
      "code": 6042,
      "name": "ExpiryTooFar",
      "msg": "Expiry must be within 24 hours from now"
    },
    {
      "code": 6043,
      "name": "ExpiryInPast",
      "msg": "Expiry must be in the future"
    },
    {
      "code": 6044,
      "name": "NonceRequired",
      "msg": "Nonce cannot be all zeros"
    },
    {
      "code": 6045,
      "name": "RotationNotFound",
      "msg": "No pending rotation exists for this vault"
    },
    {
      "code": 6046,
      "name": "RotationAlreadyFinalized",
      "msg": "Rotation has already been executed or cancelled"
    },
    {
      "code": 6047,
      "name": "RotationNotYetEffective",
      "msg": "Rotation time lock has not yet expired"
    },
    {
      "code": 6048,
      "name": "QuorumNotMetForRotation",
      "msg": "Not enough endorsements for rotation (need ceil(2n/3))"
    },
    {
      "code": 6049,
      "name": "AlreadyEndorsedRotation",
      "msg": "Validator has already endorsed this rotation"
    },
    {
      "code": 6050,
      "name": "SelfEndorsementNotAllowed",
      "msg": "Initiator cannot endorse their own rotation"
    },
    {
      "code": 6051,
      "name": "PendingRotationExists",
      "msg": "A pending rotation already exists for this vault"
    },
    {
      "code": 6052,
      "name": "PingIntervalNotElapsed",
      "msg": "Ping interval has not yet elapsed"
    },
    {
      "code": 6053,
      "name": "AlgorithmNotSupported",
      "msg": "Encryption algorithm is not supported"
    },
    {
      "code": 6054,
      "name": "TooManyStorageUris",
      "msg": "Storage URIs exceed the maximum count"
    },
    {
      "code": 6055,
      "name": "TooManyShardHolders",
      "msg": "Shard holders exceed the maximum count"
    },
    {
      "code": 6056,
      "name": "NonceAlreadyUsed",
      "msg": "This nonce has already been used for this vault"
    },
    {
      "code": 6057,
      "name": "NotAuthorizedToCreate",
      "msg": "Only the registry admin or subject's recovery wallet can create a vault"
    },
    {
      "code": 6058,
      "name": "NotAuthorizedToCancel",
      "msg": "Only the admin or initiator can cancel a rotation"
    },
    {
      "code": 6059,
      "name": "NewThresholdExceedsHolders",
      "msg": "New threshold exceeds the number of new shard holders"
    }
  ],
  "types": [
    {
      "name": "Attestation",
      "docs": [
        "An on-chain attestation that binds a set of off-chain documents/data to a",
        "parcel and records *who* (which wallets) must validate a transaction.",
        "",
        "PDA: `[\"attestation\", parcel, specifier]`. The heavy payload \u2014 actual",
        "documents and per-validator Ed25519 signatures \u2014 lives off-chain, but it is",
        "anchored here by `content_hash`, and each validator's public key is recorded",
        "so that any signature can be independently verified against this list."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "specifier",
            "docs": [
              "32-byte specifier (e.g. sha256 over the artifact/signing-session id)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "content_hash",
            "docs": [
              "sha-256 over the off-chain payload (documents, deed, survey, ...)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "required",
            "docs": [
              "Required threshold of validator signatures to consider this validated."
            ],
            "type": "u8"
          },
          {
            "name": "count",
            "docs": [
              "Number of validator keys currently registered (<= MAX_VALIDATORS)."
            ],
            "type": "u8"
          },
          {
            "name": "version",
            "docs": [
              "Monotonic rotation counter. Each rotate_validators bumps it so a",
              "reconstituted validator set is provably newer than the previous one."
            ],
            "type": "u8"
          },
          {
            "name": "created_at",
            "type": "i64"
          },
          {
            "name": "updated_at",
            "type": "i64"
          },
          {
            "name": "validators",
            "type": {
              "array": [
                "pubkey",
                8
              ]
            }
          }
        ]
      }
    },
    {
      "name": "Attested",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "specifier",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "content_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "required",
            "type": "u8"
          },
          {
            "name": "count",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "Identity",
      "docs": [
        "Binds a person (via a hashed identity credential) to a wallet the person",
        "actually holds, plus a recovery wallet. This is the resolvable on-chain link",
        "behind \"who owns this.\" A provisioned wallet is exported to the person; the",
        "program only ever sees the public keys.",
        "",
        "PDA: `[\"identity\", identity_hash]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity_hash",
            "docs": [
              "32-byte hash over the person's identity credential (e.g. national ID),",
              "so the credential itself never lives on-chain."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "owner",
            "docs": [
              "The active wallet acting on behalf of this identity."
            ],
            "type": "pubkey"
          },
          {
            "name": "recovery",
            "docs": [
              "A separate wallet the person also controls (backup / recovery). Used to",
              "request a recovery passation if the main key is lost."
            ],
            "type": "pubkey"
          },
          {
            "name": "parcel_count",
            "docs": [
              "Number of parcels currently owned by this identity."
            ],
            "type": "u16"
          },
          {
            "name": "created_at",
            "type": "i64"
          },
          {
            "name": "updated_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "IdentityBound",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "identity_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "recovery",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "InfrastructureUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "flags",
            "type": "u16"
          },
          {
            "name": "access_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "Parcel",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "geometry_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "rights_count",
            "docs": [
              "Monotonic nonce for the parcel's Rights PDAs. Never decremented."
            ],
            "type": "u8"
          },
          {
            "name": "infrastructure_flags",
            "type": "u16"
          },
          {
            "name": "access_hash",
            "docs": [
              "sha-256 canonical digest over the off-chain infra/access validation",
              "(parcel id, flags, reachability metrics). Tamper-evidence anchor."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "created_at",
            "type": "i64"
          },
          {
            "name": "updated_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ParcelAttached",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "ParcelForfeited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "case_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "from",
            "type": "pubkey"
          },
          {
            "name": "to",
            "type": "pubkey"
          },
          {
            "name": "threshold",
            "type": "u8"
          },
          {
            "name": "present",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ParcelRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "owner",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "ParcelTransferred",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "from",
            "type": "pubkey"
          },
          {
            "name": "to",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "RightGranted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "rights_kind",
            "type": "u8"
          },
          {
            "name": "holder",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "RightRevoked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "rights_kind",
            "type": "u8"
          },
          {
            "name": "holder",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "Rights",
      "docs": [
        "A right attached to a parcel (ownership, usage, easement, ...).",
        "",
        "PDA: `[\"rights\", parcel, nonce]`. One or more Rights may exist per parcel;",
        "`nonce` is allocated from `Parcel::rights_count` at grant time."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "rights_kind",
            "type": "u8"
          },
          {
            "name": "holder",
            "docs": [
              "Party holding the right."
            ],
            "type": "pubkey"
          },
          {
            "name": "granter",
            "docs": [
              "Party who granted the right (invariably the parcel owner)."
            ],
            "type": "pubkey"
          },
          {
            "name": "created_at",
            "type": "i64"
          },
          {
            "name": "expires_at",
            "docs": [
              "Unix timestamp; 0 means no expiration."
            ],
            "type": "i64"
          },
          {
            "name": "notes",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "RotationEndorsed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "new_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "endorsements_count",
            "type": "u8"
          },
          {
            "name": "required",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ShardPinged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "pinged_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ShardRotationCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "new_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "cancelled_by",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "ShardRotationExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "new_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "new_version",
            "type": "u32"
          },
          {
            "name": "new_threshold",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ShardRotationInitiated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "old_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "new_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "initiated_by",
            "type": "pubkey"
          },
          {
            "name": "effective_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "Succession",
      "docs": [
        "An in-flight passation of wallet control, gated by BOTH a configurable grace",
        "period AND a minimum number of validator endorsements (so a stolen wallet",
        "can't seize land) before it can be claimed.",
        "",
        "PDA: `[\"succession\", identity, successor]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "docs": [
              "The Identity whose control is being passed."
            ],
            "type": "pubkey"
          },
          {
            "name": "successor",
            "docs": [
              "The wallet that will take over once gated."
            ],
            "type": "pubkey"
          },
          {
            "name": "kind",
            "docs": [
              "succession_kind."
            ],
            "type": "u8"
          },
          {
            "name": "requested_at",
            "type": "i64"
          },
          {
            "name": "effective_at",
            "docs": [
              "effective = requested_at + grace_secs. Claim only allowed after this",
              "AND validations_count >= required."
            ],
            "type": "i64"
          },
          {
            "name": "grace_secs",
            "docs": [
              "Configurable per-request grace (0 => DEFAULT_SUCCESSION_GRACE_SECS)."
            ],
            "type": "i64"
          },
          {
            "name": "required",
            "docs": [
              "Number of validator endorsements required before claim (>= MIN, <= count)."
            ],
            "type": "u8"
          },
          {
            "name": "validations_count",
            "docs": [
              "Number of endorsements collected so far."
            ],
            "type": "u8"
          },
          {
            "name": "validators",
            "docs": [
              "Declared local-authority validator set acting as testifiers."
            ],
            "type": {
              "array": [
                "pubkey",
                8
              ]
            }
          }
        ]
      }
    },
    {
      "name": "SuccessionCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "successor",
            "type": "pubkey"
          },
          {
            "name": "kind",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "SuccessionClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "from",
            "type": "pubkey"
          },
          {
            "name": "to",
            "type": "pubkey"
          },
          {
            "name": "kind",
            "type": "u8"
          },
          {
            "name": "parcels_repointed",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "SuccessionEndorsed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "successor",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "validations_count",
            "type": "u8"
          },
          {
            "name": "required",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "SuccessionRequested",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "successor",
            "type": "pubkey"
          },
          {
            "name": "kind",
            "type": "u8"
          },
          {
            "name": "grace_secs",
            "type": "i64"
          },
          {
            "name": "required",
            "type": "u8"
          },
          {
            "name": "count",
            "type": "u8"
          },
          {
            "name": "effective_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ValidatorsRotated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "specifier",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "required",
            "type": "u8"
          },
          {
            "name": "count",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "VaultAccessAuthorized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "subject",
            "type": "pubkey"
          },
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "purpose",
            "type": "string"
          },
          {
            "name": "validators",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "off_chain_nonce",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "expiry",
            "type": "i64"
          },
          {
            "name": "block_time",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "VaultCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "subject",
            "type": "pubkey"
          },
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "algorithm_id",
            "type": "u8"
          },
          {
            "name": "threshold",
            "type": "u8"
          },
          {
            "name": "holder_count",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "VaultRecord",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "subject",
            "type": "pubkey"
          },
          {
            "name": "ciphertext_cid",
            "type": "string"
          },
          {
            "name": "ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "algorithm_id",
            "type": "u8"
          },
          {
            "name": "storage_uris",
            "type": {
              "vec": "string"
            }
          },
          {
            "name": "shard_holders",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "threshold",
            "type": "u8"
          },
          {
            "name": "version",
            "type": "u32"
          },
          {
            "name": "last_ping_at",
            "type": "i64"
          },
          {
            "name": "created_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "VaultShardRotation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "old_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "new_ciphertext_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "new_shard_holders",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "new_threshold",
            "type": "u8"
          },
          {
            "name": "initiated_by",
            "type": "pubkey"
          },
          {
            "name": "endorsements",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "required_endorsements",
            "type": "u8"
          },
          {
            "name": "initiated_at",
            "type": "i64"
          },
          {
            "name": "effective_at",
            "type": "i64"
          },
          {
            "name": "status",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "AuthorityRegistry",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "validators",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "mode",
            "type": "u8"
          },
          {
            "name": "created_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ConsensusFlipped",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "required_endorsements",
            "type": "u8"
          },
          {
            "name": "validator_count",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "DocumentAnchor",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "attestation",
            "type": "pubkey"
          },
          {
            "name": "cid",
            "type": "string"
          },
          {
            "name": "content_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "category",
            "type": "string"
          },
          {
            "name": "registered_by",
            "type": "pubkey"
          },
          {
            "name": "registered_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "DocumentRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "attestation",
            "type": "pubkey"
          },
          {
            "name": "cid",
            "type": "string"
          },
          {
            "name": "content_hash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "category",
            "type": "string"
          },
          {
            "name": "registered_by",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "RegistryCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "mode",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ValidatorAdded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "added_by",
            "type": "pubkey"
          },
          {
            "name": "mode",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ValidatorEndorsed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "proposed",
            "type": "pubkey"
          },
          {
            "name": "endorser",
            "type": "pubkey"
          },
          {
            "name": "endorsements_count",
            "type": "u8"
          },
          {
            "name": "required",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ValidatorEndorsement",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "proposed",
            "type": "pubkey"
          },
          {
            "name": "endorsers",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "added_at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ValidatorRemoved",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "mode",
            "type": "u8"
          }
        ]
      }
    }
  ]
}
;
