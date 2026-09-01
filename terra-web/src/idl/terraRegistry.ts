/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/terra_registry.json`.
 */
export type TerraRegistry = {
  "address": "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage",
  "metadata": {
    "name": "terraRegistry",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "attachParcel",
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
                "account": "parcel"
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
                "path": "identity.identityHash",
                "account": "identity"
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
                "account": "parcel"
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
          "name": "systemProgram",
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
          "name": "contentHash",
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
      "name": "authorizeVaultAccess",
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
          "name": "vaultRecord",
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
          "name": "systemProgram",
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
          "name": "offChainNonce",
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
      "name": "bindIdentity",
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
                "path": "identityHash"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "identityHash",
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
      "name": "cancelShardRotation",
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
          "name": "vaultRecord"
        },
        {
          "name": "canceller",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "cancelSuccession",
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
                "path": "identity.identityHash",
                "account": "identity"
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
                "account": "succession"
              },
              {
                "kind": "account",
                "path": "succession.successor",
                "account": "succession"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "claimSuccession",
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
                "path": "identity.identityHash",
                "account": "identity"
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
                "account": "succession"
              },
              {
                "kind": "account",
                "path": "succession.successor",
                "account": "succession"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "createVault",
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
          "name": "vaultRecord",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "ciphertextCid",
          "type": "string"
        },
        {
          "name": "ciphertextHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "algorithmId",
          "type": "u8"
        },
        {
          "name": "storageUris",
          "type": {
            "vec": "string"
          }
        },
        {
          "name": "shardHolders",
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
      "name": "endorseShardRotation",
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
          "name": "vaultRecord"
        },
        {
          "name": "validator",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "endorseSuccession",
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
                "path": "identity.identityHash",
                "account": "identity"
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
                "account": "succession"
              },
              {
                "kind": "account",
                "path": "succession.successor",
                "account": "succession"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "executeShardRotation",
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
          "name": "vaultRecord",
          "writable": true
        },
        {
          "name": "initiator",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "grantRight",
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
                "account": "parcel"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "nonce",
          "type": "u8"
        },
        {
          "name": "rightsKind",
          "type": "u8"
        },
        {
          "name": "holder",
          "type": "pubkey"
        },
        {
          "name": "expiresAt",
          "type": "i64"
        },
        {
          "name": "notes",
          "type": "string"
        }
      ]
    },
    {
      "name": "initiateShardRotation",
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
                "path": "vaultRecord"
              },
              {
                "kind": "arg",
                "path": "newCiphertextHash"
              }
            ]
          }
        },
        {
          "name": "vaultRecord",
          "writable": true
        },
        {
          "name": "initiator",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "newCiphertextHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "newShardHolders",
          "type": {
            "vec": "pubkey"
          }
        },
        {
          "name": "newThreshold",
          "type": "u8"
        }
      ]
    },
    {
      "name": "judicialForfeiture",
      "docs": [
        "Force-transfer a parcel's ownership away from a non-compliant owner, per",
        "a court order. This is deliberately heavier than a normal transfer:",
        "at least `MIN_FORFEIT_VALIDATORS` (2) of the declared validators must",
        "sign this transaction themselves, and the order is bound to a",
        "`case_hash` (e.g. SHA-256 of the court order document) for auditability.",
        "",
        "This is how validators collectively inform the chain that land no longer",
        "belongs to someone who refuses to release it — e.g. repossession by a",
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
                "account": "parcel"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "caseHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "newOwner",
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
      "name": "pingShard",
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
          "name": "vaultRecord",
          "writable": true
        },
        {
          "name": "validator",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "registerParcel",
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
          "name": "systemProgram",
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
          "name": "geometryHash",
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
      "name": "requestSuccession",
      "docs": [
        "Request a wallet passation (succession, recovery, or deliberate control",
        "transfer). A Succession account is created and becomes effective only",
        "after the grace period — within which the original owner can cancel.",
        "",
        "Authorized by the current `owner` for kind TRANSFER, or by the `owner`",
        "OR the `recovery` wallet for kind RECOVERY/SUCCESSOR.",
        "",
        "`grace_secs` lets the requester choose the window (0 => default 30d),",
        "clamped to [MIN, MAX]. `required_validations` is the number of declared",
        "local validators that must endorse the passation before it can be",
        "claimed (>= 1) — so a stolen wallet can't seize land alone.",
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
                "path": "identity.identityHash",
                "account": "identity"
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
          "name": "systemProgram",
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
          "name": "graceSecs",
          "type": "i64"
        },
        {
          "name": "requiredValidations",
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
      "name": "revokeRight",
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
                "account": "parcel"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "nonce",
          "type": "u8"
        }
      ]
    },
    {
      "name": "rotateValidators",
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
                "account": "parcel"
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
                "account": "attestation"
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
          "name": "newRequired",
          "type": "u8"
        },
        {
          "name": "newValidators",
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
      "name": "transferParcel",
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
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "owner",
          "signer": true
        },
        {
          "name": "newOwner"
        }
      ],
      "args": []
    },
    {
      "name": "updateInfrastructure",
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
                "account": "parcel"
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
          "name": "accessHash",
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
      "name": "updateStatus",
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
                "account": "parcel"
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
      "name": "attestation",
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
      "name": "identity",
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
      "name": "parcel",
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
      "name": "rights",
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
      "name": "succession",
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
      "name": "vaultRecord",
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
      "name": "vaultShardRotation",
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
    }
  ],
  "events": [
    {
      "name": "attested",
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
      "name": "identityBound",
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
      "name": "infrastructureUpdated",
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
      "name": "parcelAttached",
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
      "name": "parcelForfeited",
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
      "name": "parcelRegistered",
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
      "name": "parcelTransferred",
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
      "name": "rightGranted",
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
      "name": "rightRevoked",
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
      "name": "rotationEndorsed",
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
      "name": "shardPinged",
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
      "name": "shardRotationCancelled",
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
      "name": "shardRotationExecuted",
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
      "name": "shardRotationInitiated",
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
      "name": "successionCancelled",
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
      "name": "successionClaimed",
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
      "name": "successionEndorsed",
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
      "name": "successionRequested",
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
      "name": "validatorsRotated",
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
      "name": "vaultAccessAuthorized",
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
      "name": "vaultCreated",
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
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidId",
      "msg": "Parcel id cannot be all zeros"
    },
    {
      "code": 6001,
      "name": "emptyName",
      "msg": "Parcel name cannot be empty"
    },
    {
      "code": 6002,
      "name": "emptyGeometryHash",
      "msg": "Geometry hash is required"
    },
    {
      "code": 6003,
      "name": "notOwner",
      "msg": "Only the current owner can perform this action"
    },
    {
      "code": 6004,
      "name": "invalidStatus",
      "msg": "Invalid parcel status"
    },
    {
      "code": 6005,
      "name": "invalidRightKind",
      "msg": "Invalid right kind"
    },
    {
      "code": 6006,
      "name": "invalidNonce",
      "msg": "Nonce does not match the parcel's rights_count"
    },
    {
      "code": 6007,
      "name": "rightsLimitExceeded",
      "msg": "Rights limit reached"
    },
    {
      "code": 6008,
      "name": "notesTooLong",
      "msg": "Notes exceed the maximum length of 128"
    },
    {
      "code": 6009,
      "name": "invalidExpiry",
      "msg": "Expiry must be in the future"
    },
    {
      "code": 6010,
      "name": "notAuthorized",
      "msg": "Not authorized to perform this action"
    },
    {
      "code": 6011,
      "name": "invalidInfrastructureFlags",
      "msg": "Invalid infrastructure flags"
    },
    {
      "code": 6012,
      "name": "emptyAccessHash",
      "msg": "Access hash is required"
    },
    {
      "code": 6013,
      "name": "emptySpecifier",
      "msg": "Attestation specifier is required"
    },
    {
      "code": 6014,
      "name": "emptyContentHash",
      "msg": "Content hash is required"
    },
    {
      "code": 6015,
      "name": "noValidators",
      "msg": "Attestation requires at least one validator"
    },
    {
      "code": 6016,
      "name": "invalidThreshold",
      "msg": "Required threshold exceeds the number of validators"
    },
    {
      "code": 6017,
      "name": "emptyIdentityHash",
      "msg": "Identity hash is required"
    },
    {
      "code": 6018,
      "name": "emptyRecovery",
      "msg": "Recovery wallet is required"
    },
    {
      "code": 6019,
      "name": "identityMismatch",
      "msg": "Identity owner does not match the parcel owner"
    },
    {
      "code": 6020,
      "name": "emptySuccessor",
      "msg": "Successor wallet is required"
    },
    {
      "code": 6021,
      "name": "invalidSuccessionKind",
      "msg": "Invalid succession kind"
    },
    {
      "code": 6022,
      "name": "successorIsOwner",
      "msg": "Successor must differ from the current owner"
    },
    {
      "code": 6023,
      "name": "successionAlreadyEffective",
      "msg": "Succession has already become effective"
    },
    {
      "code": 6024,
      "name": "notSuccessor",
      "msg": "Only the named successor may claim this succession"
    },
    {
      "code": 6025,
      "name": "successionNotYetEffective",
      "msg": "Succession is not yet effective"
    },
    {
      "code": 6026,
      "name": "attestationMismatch",
      "msg": "Attestation does not belong to this parcel"
    },
    {
      "code": 6027,
      "name": "insufficientValidations",
      "msg": "Succession requires validator endorsements before it can be claimed"
    },
    {
      "code": 6028,
      "name": "notValidator",
      "msg": "Signing wallet is not a declared validator for this succession"
    },
    {
      "code": 6029,
      "name": "validationLimitReached",
      "msg": "No more validators may endorse this succession (limit reached)"
    },
    {
      "code": 6030,
      "name": "emptyCaseHash",
      "msg": "Court case hash is required"
    },
    {
      "code": 6031,
      "name": "emptyNewOwner",
      "msg": "New forfeiture owner is required"
    },
    {
      "code": 6032,
      "name": "insufficientValidatorSigners",
      "msg": "Not enough validator signers to forfeit this parcel"
    },
    {
      "code": 6033,
      "name": "ownerCannotSelfForfeit",
      "msg": "The current owner cannot self-forfeit their own parcel"
    },
    {
      "code": 6034,
      "name": "vaultAlreadyExists",
      "msg": "Vault already exists for this subject"
    },
    {
      "code": 6035,
      "name": "vaultNotFound",
      "msg": "Vault not found"
    },
    {
      "code": 6036,
      "name": "thresholdExceedsHolders",
      "msg": "Threshold exceeds the number of shard holders"
    },
    {
      "code": 6037,
      "name": "notShardHolder",
      "msg": "Signer is not a shard holder for this vault"
    },
    {
      "code": 6038,
      "name": "notActiveValidator",
      "msg": "Signer is not an active validator in this vault"
    },
    {
      "code": 6039,
      "name": "ciphertextHashRequired",
      "msg": "Ciphertext hash cannot be all zeros"
    },
    {
      "code": 6040,
      "name": "cidRequired",
      "msg": "Ciphertext CID cannot be empty"
    },
    {
      "code": 6041,
      "name": "expiryTooFar",
      "msg": "Expiry must be within 24 hours from now"
    },
    {
      "code": 6042,
      "name": "expiryInPast",
      "msg": "Expiry must be in the future"
    },
    {
      "code": 6043,
      "name": "nonceRequired",
      "msg": "Nonce cannot be all zeros"
    },
    {
      "code": 6044,
      "name": "rotationNotFound",
      "msg": "No pending rotation exists for this vault"
    },
    {
      "code": 6045,
      "name": "rotationAlreadyFinalized",
      "msg": "Rotation has already been executed or cancelled"
    },
    {
      "code": 6046,
      "name": "rotationNotYetEffective",
      "msg": "Rotation time lock has not yet expired"
    },
    {
      "code": 6047,
      "name": "quorumNotMetForRotation",
      "msg": "Not enough endorsements for rotation (need ceil(2n/3))"
    },
    {
      "code": 6048,
      "name": "alreadyEndorsedRotation",
      "msg": "Validator has already endorsed this rotation"
    },
    {
      "code": 6049,
      "name": "selfEndorsementNotAllowed",
      "msg": "Initiator cannot endorse their own rotation"
    },
    {
      "code": 6050,
      "name": "pendingRotationExists",
      "msg": "A pending rotation already exists for this vault"
    },
    {
      "code": 6051,
      "name": "pingIntervalNotElapsed",
      "msg": "Ping interval has not yet elapsed"
    },
    {
      "code": 6052,
      "name": "algorithmNotSupported",
      "msg": "Encryption algorithm is not supported"
    },
    {
      "code": 6053,
      "name": "tooManyStorageUris",
      "msg": "Storage URIs exceed the maximum count"
    },
    {
      "code": 6054,
      "name": "tooManyShardHolders",
      "msg": "Shard holders exceed the maximum count"
    },
    {
      "code": 6055,
      "name": "nonceAlreadyUsed",
      "msg": "This nonce has already been used for this vault"
    },
    {
      "code": 6056,
      "name": "notAuthorizedToCreate",
      "msg": "Only the registry admin or subject's recovery wallet can create a vault"
    },
    {
      "code": 6057,
      "name": "notAuthorizedToCancel",
      "msg": "Only the admin or initiator can cancel a rotation"
    },
    {
      "code": 6058,
      "name": "newThresholdExceedsHolders",
      "msg": "New threshold exceeds the number of new shard holders"
    }
  ],
  "types": [
    {
      "name": "attestation",
      "docs": [
        "An on-chain attestation that binds a set of off-chain documents/data to a",
        "parcel and records *who* (which wallets) must validate a transaction.",
        "",
        "PDA: `[\"attestation\", parcel, specifier]`. The heavy payload — actual",
        "documents and per-validator Ed25519 signatures — lives off-chain, but it is",
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
            "name": "contentHash",
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
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
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
      "name": "attested",
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
            "name": "contentHash",
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
      "name": "identity",
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
            "name": "identityHash",
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
            "name": "parcelCount",
            "docs": [
              "Number of parcels currently owned by this identity."
            ],
            "type": "u16"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "identityBound",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "identityHash",
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
      "name": "infrastructureUpdated",
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
            "name": "accessHash",
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
      "name": "parcel",
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
            "name": "geometryHash",
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
            "name": "rightsCount",
            "docs": [
              "Monotonic nonce for the parcel's Rights PDAs. Never decremented."
            ],
            "type": "u8"
          },
          {
            "name": "infrastructureFlags",
            "type": "u16"
          },
          {
            "name": "accessHash",
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
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "parcelAttached",
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
      "name": "parcelForfeited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "caseHash",
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
      "name": "parcelRegistered",
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
      "name": "parcelTransferred",
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
      "name": "rightGranted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "rightsKind",
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
      "name": "rightRevoked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "rightsKind",
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
      "name": "rights",
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
            "name": "rightsKind",
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
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "expiresAt",
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
      "name": "rotationEndorsed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "newCiphertextHash",
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
            "name": "endorsementsCount",
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
      "name": "shardPinged",
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
            "name": "pingedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "shardRotationCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "newCiphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "cancelledBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "shardRotationExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "newCiphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "newVersion",
            "type": "u32"
          },
          {
            "name": "newThreshold",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "shardRotationInitiated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "oldCiphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "newCiphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "initiatedBy",
            "type": "pubkey"
          },
          {
            "name": "effectiveAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "succession",
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
            "name": "requestedAt",
            "type": "i64"
          },
          {
            "name": "effectiveAt",
            "docs": [
              "effective = requested_at + grace_secs. Claim only allowed after this",
              "AND validations_count >= required."
            ],
            "type": "i64"
          },
          {
            "name": "graceSecs",
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
            "name": "validationsCount",
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
      "name": "successionCancelled",
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
      "name": "successionClaimed",
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
            "name": "parcelsRepointed",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "successionEndorsed",
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
            "name": "validationsCount",
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
      "name": "successionRequested",
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
            "name": "graceSecs",
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
            "name": "effectiveAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorsRotated",
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
      "name": "vaultAccessAuthorized",
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
            "name": "offChainNonce",
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
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "vaultCreated",
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
            "name": "ciphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "algorithmId",
            "type": "u8"
          },
          {
            "name": "threshold",
            "type": "u8"
          },
          {
            "name": "holderCount",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "vaultRecord",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "subject",
            "type": "pubkey"
          },
          {
            "name": "ciphertextCid",
            "type": "string"
          },
          {
            "name": "ciphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "algorithmId",
            "type": "u8"
          },
          {
            "name": "storageUris",
            "type": {
              "vec": "string"
            }
          },
          {
            "name": "shardHolders",
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
            "name": "lastPingAt",
            "type": "i64"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "vaultShardRotation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "oldCiphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "newCiphertextHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "newShardHolders",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "newThreshold",
            "type": "u8"
          },
          {
            "name": "initiatedBy",
            "type": "pubkey"
          },
          {
            "name": "endorsements",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "requiredEndorsements",
            "type": "u8"
          },
          {
            "name": "initiatedAt",
            "type": "i64"
          },
          {
            "name": "effectiveAt",
            "type": "i64"
          },
          {
            "name": "status",
            "type": "u8"
          }
        ]
      }
    }
  ]
};
