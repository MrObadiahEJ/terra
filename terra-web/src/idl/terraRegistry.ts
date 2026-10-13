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
      "name": "acceptEscrow",
      "discriminator": [
        193,
        2,
        224,
        245,
        36,
        116,
        65,
        154
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "seller",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "addEvidence",
      "discriminator": [
        205,
        19,
        129,
        228,
        117,
        97,
        33,
        49
      ],
      "accounts": [
        {
          "name": "evidence",
          "writable": true
        },
        {
          "name": "claim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "submitter",
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
          "name": "evidenceType",
          "type": "u8"
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
          "name": "storageReference",
          "type": "string"
        },
        {
          "name": "observedAt",
          "type": "i64"
        }
      ]
    },
    {
      "name": "addEvidenceArtifact",
      "discriminator": [
        168,
        32,
        22,
        198,
        143,
        156,
        44,
        153
      ],
      "accounts": [
        {
          "name": "artifact",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  105,
                  100,
                  101,
                  110,
                  99,
                  101,
                  95,
                  97,
                  114,
                  116,
                  105,
                  102,
                  97,
                  99,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "manifest"
              },
              {
                "kind": "arg",
                "path": "artifactIndex"
              }
            ]
          }
        },
        {
          "name": "manifest",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  105,
                  100,
                  101,
                  110,
                  99,
                  101,
                  95,
                  109,
                  97,
                  110,
                  105,
                  102,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "manifest.taskId",
                "account": "evidenceManifest"
              },
              {
                "kind": "account",
                "path": "manifest.submitter",
                "account": "evidenceManifest"
              },
              {
                "kind": "account",
                "path": "manifest.nonce",
                "account": "evidenceManifest"
              }
            ]
          }
        },
        {
          "name": "task",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "manifest.taskId",
                "account": "evidenceManifest"
              }
            ]
          }
        },
        {
          "name": "submitter",
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
          "name": "artifactIndex",
          "type": "u16"
        },
        {
          "name": "kind",
          "type": "u8"
        },
        {
          "name": "source",
          "type": "u8"
        },
        {
          "name": "provenance",
          "type": "u8"
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
          "name": "storageReference",
          "type": "string"
        }
      ]
    },
    {
      "name": "addSecondValidator",
      "discriminator": [
        117,
        163,
        217,
        11,
        213,
        152,
        156,
        140
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "sponsor",
          "docs": [
            "Must be validator #1 (the first validator in the registry)."
          ],
          "signer": true
        },
        {
          "name": "candidate"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "addTaskRequirement",
      "discriminator": [
        181,
        37,
        124,
        255,
        238,
        152,
        220,
        215
      ],
      "accounts": [
        {
          "name": "requirement",
          "writable": true
        },
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "requester",
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
          "name": "reqIndex",
          "type": "u8"
        },
        {
          "name": "capabilityCode",
          "type": "u8"
        },
        {
          "name": "minReputation",
          "type": "u16"
        },
        {
          "name": "minTier",
          "type": "u8"
        },
        {
          "name": "jurisdiction",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        },
        {
          "name": "radiusM",
          "type": "u32"
        },
        {
          "name": "centerLatE7",
          "type": "i32"
        },
        {
          "name": "centerLonE7",
          "type": "i32"
        },
        {
          "name": "independenceBps",
          "type": "u16"
        },
        {
          "name": "confidenceTargetBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "addThirdValidator",
      "discriminator": [
        106,
        119,
        134,
        212,
        149,
        208,
        154,
        41
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "candidate"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "candidate",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "addValidatorToRegistry",
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
          "name": "adminSigner",
          "docs": [
            "Admin signer — required in bootstrap mode, ignored in peer-consensus",
            "(the endorsement account carries authority instead)."
          ],
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
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator"
        },
        {
          "name": "systemProgram",
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
      "name": "adjudicateDispute",
      "discriminator": [
        0,
        67,
        102,
        188,
        15,
        158,
        179,
        75
      ],
      "accounts": [
        {
          "name": "dispute",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  112,
                  117,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "account",
                "path": "dispute.caseHash",
                "account": "dispute"
              }
            ]
          }
        },
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
          "name": "registry",
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
          "name": "authority",
          "docs": [
            "Court authority or designated adjudicator — must be registry admin."
          ],
          "signer": true
        }
      ],
      "args": [
        {
          "name": "outcome",
          "type": "u8"
        },
        {
          "name": "newOwner",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "adminVerifyValidatorCapability",
      "discriminator": [
        136,
        212,
        150,
        72,
        59,
        178,
        184,
        225
      ],
      "accounts": [
        {
          "name": "capability",
          "writable": true
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "profile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "capabilityCode",
          "type": "u8"
        },
        {
          "name": "level",
          "type": "u8"
        },
        {
          "name": "evidenceHash",
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
      "name": "allocateCountry",
      "discriminator": [
        110,
        200,
        50,
        85,
        16,
        27,
        168,
        249
      ],
      "accounts": [
        {
          "name": "worldRegistry",
          "writable": true
        },
        {
          "name": "admin",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "countryCode",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        },
        {
          "name": "approvedAdmin",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "amalgamateParcels",
      "discriminator": [
        169,
        18,
        137,
        101,
        59,
        136,
        207,
        162
      ],
      "accounts": [
        {
          "name": "resultParcel",
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
                "path": "resultParcel.id",
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "resultOwnership",
          "docs": [
            "Ownership right of the result parcel — authority must control it."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "resultParcel"
              }
            ]
          }
        },
        {
          "name": "sourceParcel",
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
                "path": "sourceParcel.id",
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "sourceOwnership",
          "docs": [
            "Ownership right of the source parcel — authority must control it too."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "sourceParcel"
              }
            ]
          }
        },
        {
          "name": "amalgamationRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  109,
                  97,
                  108,
                  103,
                  97,
                  109,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "resultParcel"
              },
              {
                "kind": "account",
                "path": "sourceParcel"
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
          "name": "newGeometryHash",
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
      "name": "appendGeometryVersion",
      "discriminator": [
        130,
        154,
        224,
        216,
        162,
        208,
        164,
        163
      ],
      "accounts": [
        {
          "name": "asset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "asset.parcel",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "geometryVersion",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  101,
                  111,
                  109,
                  101,
                  116,
                  114,
                  121,
                  95,
                  118,
                  101,
                  114,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "asset"
              },
              {
                "kind": "account",
                "path": "asset.geometryVersionCount",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "payer",
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
          "name": "geometryHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "source",
          "type": "u8"
        },
        {
          "name": "dimension",
          "type": "u8"
        },
        {
          "name": "storageReference",
          "type": "string"
        },
        {
          "name": "elevationMinMm",
          "type": "i32"
        },
        {
          "name": "elevationMaxMm",
          "type": "i32"
        },
        {
          "name": "elevationSource",
          "type": "u8"
        }
      ]
    },
    {
      "name": "appendGeometryVersionFromEvidence",
      "docs": [
        "Anchor a geometry version from an evidence manifest (RFC-013 Phase B)."
      ],
      "discriminator": [
        181,
        10,
        220,
        112,
        151,
        247,
        247,
        243
      ],
      "accounts": [
        {
          "name": "asset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "asset.parcel",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "geometryVersion",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  101,
                  111,
                  109,
                  101,
                  116,
                  114,
                  121,
                  95,
                  118,
                  101,
                  114,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "asset"
              },
              {
                "kind": "account",
                "path": "asset.geometryVersionCount",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "manifest",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  105,
                  100,
                  101,
                  110,
                  99,
                  101,
                  95,
                  109,
                  97,
                  110,
                  105,
                  102,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "manifest.taskId",
                "account": "evidenceManifest"
              },
              {
                "kind": "account",
                "path": "manifest.submitter",
                "account": "evidenceManifest"
              },
              {
                "kind": "account",
                "path": "manifest.nonce",
                "account": "evidenceManifest"
              }
            ]
          }
        },
        {
          "name": "task",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "manifest.taskId",
                "account": "evidenceManifest"
              }
            ]
          }
        },
        {
          "name": "payer",
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
          "name": "geometryHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "source",
          "type": "u8"
        },
        {
          "name": "dimension",
          "type": "u8"
        },
        {
          "name": "storageReference",
          "type": "string"
        },
        {
          "name": "elevationMinMm",
          "type": "i32"
        },
        {
          "name": "elevationMaxMm",
          "type": "i32"
        },
        {
          "name": "elevationSource",
          "type": "u8"
        }
      ]
    },
    {
      "name": "assignTaskValidator",
      "discriminator": [
        217,
        224,
        198,
        65,
        216,
        139,
        13,
        23
      ],
      "accounts": [
        {
          "name": "assignment",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  97,
                  115,
                  115,
                  105,
                  103,
                  110,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "validator"
        },
        {
          "name": "requester",
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
          "name": "taskId",
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
      "name": "attachParcel",
      "docs": [
        "Attach a parcel to an identity (the person behind its holder wallet).",
        "Only the current ownership holder may do this, and only for an identity",
        "whose owner wallet matches. The identity link is provenance — it never",
        "becomes the ownership record."
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "identity"
        },
        {
          "name": "owner",
          "signer": true
        }
      ],
      "args": []
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
      "name": "bindCrossBorderIdentity",
      "discriminator": [
        221,
        72,
        207,
        129,
        248,
        66,
        1,
        35
      ],
      "accounts": [
        {
          "name": "binding",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
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
                "path": "jurisdiction"
              },
              {
                "kind": "arg",
                "path": "identityHash"
              }
            ]
          }
        },
        {
          "name": "identity"
        },
        {
          "name": "jurisdiction",
          "writable": true
        },
        {
          "name": "prover",
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
          "name": "credentialCommitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "proofData",
          "type": "bytes"
        },
        {
          "name": "nullifierNonce",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "expiresAt",
          "type": "i64"
        },
        {
          "name": "identityHash",
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
      "name": "bootstrapSelfProclaim",
      "discriminator": [
        150,
        57,
        45,
        172,
        227,
        150,
        140,
        24
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "candidate",
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
          "name": "countryCode",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        }
      ]
    },
    {
      "name": "cancelDispute",
      "discriminator": [
        23,
        155,
        220,
        94,
        76,
        141,
        231,
        124
      ],
      "accounts": [
        {
          "name": "dispute",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  112,
                  117,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "account",
                "path": "dispute.caseHash",
                "account": "dispute"
              }
            ]
          }
        },
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "signer",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "cancelEscrow",
      "discriminator": [
        156,
        203,
        54,
        179,
        38,
        72,
        33,
        21
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "escrowVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "escrowRecord"
              }
            ]
          }
        },
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
          "name": "signer",
          "writable": true,
          "signer": true
        },
        {
          "name": "buyer",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
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
      "name": "cancelTask",
      "discriminator": [
        69,
        228,
        134,
        187,
        134,
        105,
        238,
        48
      ],
      "accounts": [
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "requester",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "taskId",
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
      "name": "castAppealVote",
      "discriminator": [
        76,
        236,
        68,
        226,
        134,
        226,
        237,
        93
      ],
      "accounts": [
        {
          "name": "review",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  118,
                  105,
                  101,
                  119,
                  95,
                  99,
                  97,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "review.report",
                "account": "reviewCase"
              }
            ]
          }
        },
        {
          "name": "appeal"
        },
        {
          "name": "voter",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "grant",
          "type": "bool"
        }
      ]
    },
    {
      "name": "castFraudVote",
      "discriminator": [
        55,
        195,
        39,
        206,
        219,
        91,
        112,
        78
      ],
      "accounts": [
        {
          "name": "review",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  118,
                  105,
                  101,
                  119,
                  95,
                  99,
                  97,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "review.report",
                "account": "reviewCase"
              }
            ]
          }
        },
        {
          "name": "report"
        },
        {
          "name": "voter",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "uphold",
          "type": "bool"
        }
      ]
    },
    {
      "name": "castQuorumVote",
      "discriminator": [
        27,
        98,
        166,
        85,
        214,
        186,
        65,
        210
      ],
      "accounts": [
        {
          "name": "quorumVote",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  113,
                  117,
                  111,
                  114,
                  117,
                  109,
                  95,
                  118,
                  111,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              },
              {
                "kind": "account",
                "path": "voter"
              }
            ]
          }
        },
        {
          "name": "tally",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  113,
                  117,
                  111,
                  114,
                  117,
                  109,
                  95,
                  116,
                  97,
                  108,
                  108,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              }
            ]
          }
        },
        {
          "name": "claim",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "voter",
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
          "name": "voteChoice",
          "type": "u8"
        }
      ]
    },
    {
      "name": "checkQuorumReachable",
      "discriminator": [
        236,
        95,
        184,
        254,
        87,
        254,
        2,
        204
      ],
      "accounts": [
        {
          "name": "registry",
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
        }
      ],
      "args": []
    },
    {
      "name": "claimRewards",
      "discriminator": [
        4,
        144,
        132,
        71,
        116,
        23,
        151,
        80
      ],
      "accounts": [
        {
          "name": "validatorStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "stakePool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "validator",
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
      "name": "claimSuccessionWithParcels",
      "docs": [
        "Claim a succession AND transfer every ownership right it covers in one",
        "atomic step (B6 boundary rule: identity state is only ever *composed*",
        "from this program — never written directly).",
        "",
        "The identity-side claim runs as a CPI into",
        "`terra_identity::claim_succession`, which enforces its own guards,",
        "moves `identity.owner` to the successor, closes the Succession PDA",
        "(rent to the signer) and emits `SuccessionClaimed`. Only afterwards",
        "does this handler re-point the provided registry-owned ownership",
        "Rights PDAs from the previous identity owner to the successor.",
        "",
        "`remaining_accounts` = ownership Rights PDAs (`[\"ownership\", parcel]`).",
        "Each must be held either by the previous identity-owner wallet",
        "(re-pointed to the successor) or by the Identity PDA itself (skipped —",
        "it already follows the identity). If the CPI fails, no rights are",
        "written; if a rights write fails, the CPI rolls back. Atomic."
      ],
      "discriminator": [
        233,
        141,
        78,
        83,
        199,
        38,
        185,
        110
      ],
      "accounts": [
        {
          "name": "identity",
          "docs": [
            "mutated only by the CPI into terra_identity::claim_succession."
          ],
          "writable": true
        },
        {
          "name": "succession",
          "writable": true
        },
        {
          "name": "signer",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "identityProgram",
          "docs": [
            "transaction so the CPI can resolve the callee program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "claimTask",
      "discriminator": [
        49,
        222,
        219,
        238,
        155,
        68,
        221,
        136
      ],
      "accounts": [
        {
          "name": "assignment",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  97,
                  115,
                  115,
                  105,
                  103,
                  110,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "validator",
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
          "name": "taskId",
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
      "name": "claimTaskReward",
      "discriminator": [
        105,
        212,
        24,
        245,
        14,
        124,
        23,
        131
      ],
      "accounts": [
        {
          "name": "task"
        },
        {
          "name": "assignment",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  97,
                  115,
                  115,
                  105,
                  103,
                  110,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              },
              {
                "kind": "account",
                "path": "assignment.validator",
                "account": "taskAssignment"
              }
            ]
          }
        },
        {
          "name": "taskEscrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "taskEscrow"
              }
            ]
          }
        },
        {
          "name": "coverageIncentive",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  118,
                  101,
                  114,
                  97,
                  103,
                  101,
                  95,
                  105,
                  110,
                  99,
                  101,
                  110,
                  116,
                  105,
                  118,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "rewardAllocation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  119,
                  97,
                  114,
                  100,
                  95,
                  97,
                  108,
                  108,
                  111,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "treasury",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "validator",
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
      "name": "closeVerificationSession",
      "discriminator": [
        188,
        203,
        38,
        211,
        250,
        237,
        66,
        113
      ],
      "accounts": [
        {
          "name": "session",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  115,
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
                "path": "session.claim",
                "account": "verificationSession"
              },
              {
                "kind": "account",
                "path": "session.sessionId",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "sessionTracker",
          "docs": [
            "Must match the tracker for this claim."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109,
                  95,
                  115,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110,
                  95,
                  116,
                  114,
                  97,
                  99,
                  107,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "session.claim",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "opener",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "finalStatus",
          "type": "u8"
        }
      ]
    },
    {
      "name": "confirmGenesis",
      "discriminator": [
        116,
        83,
        231,
        21,
        73,
        89,
        137,
        159
      ],
      "accounts": [
        {
          "name": "genesisRequest",
          "writable": true
        },
        {
          "name": "worldRegistry",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  111,
                  114,
                  108,
                  100,
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
          "name": "confirmer",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "confirmerCountry",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        }
      ]
    },
    {
      "name": "confirmNomination",
      "discriminator": [
        216,
        43,
        186,
        199,
        137,
        98,
        63,
        109
      ],
      "accounts": [
        {
          "name": "nomination",
          "writable": true
        },
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "confirmer",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "createClaim",
      "discriminator": [
        71,
        122,
        43,
        84,
        240,
        165,
        215,
        181
      ],
      "accounts": [
        {
          "name": "claim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "arg",
                "path": "claimId"
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
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "submitter",
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
          "name": "claimId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "claimType",
          "type": "u8"
        },
        {
          "name": "statementHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "parcelType",
          "type": "u8"
        },
        {
          "name": "region",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        }
      ]
    },
    {
      "name": "createCrossBorderBinding",
      "discriminator": [
        165,
        96,
        179,
        203,
        103,
        54,
        74,
        73
      ],
      "accounts": [
        {
          "name": "binding",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  98,
                  105,
                  110,
                  100,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "arg",
                "path": "countryMin"
              },
              {
                "kind": "arg",
                "path": "countryMax"
              }
            ]
          }
        },
        {
          "name": "jurisdictionMin"
        },
        {
          "name": "jurisdictionMax"
        },
        {
          "name": "authorityMin",
          "writable": true,
          "signer": true
        },
        {
          "name": "authorityMax",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "countryMin",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        },
        {
          "name": "countryMax",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        },
        {
          "name": "expiresAt",
          "type": "i64"
        }
      ]
    },
    {
      "name": "createEscrow",
      "discriminator": [
        253,
        215,
        165,
        116,
        36,
        108,
        68,
        80
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "escrowVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "escrowRecord"
              }
            ]
          }
        },
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "seller",
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
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "buyer",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "createGuardianClaim",
      "discriminator": [
        220,
        101,
        157,
        51,
        139,
        2,
        93,
        1
      ],
      "accounts": [
        {
          "name": "guardianClaim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  117,
                  97,
                  114,
                  100,
                  105,
                  97,
                  110,
                  95,
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              }
            ]
          }
        },
        {
          "name": "claim",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "identity"
        },
        {
          "name": "registry",
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
          "name": "caller",
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
          "name": "caseHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "guardianType",
          "type": "u8"
        }
      ]
    },
    {
      "name": "createRegistry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "createSpatialSnapshot",
      "docs": [
        "Freeze a point-in-time copy of the asset's spatial indexes",
        "(RFC-012 §5 `SpatialSnapshot`, RFC-013 §8.5 — snapshots copy the",
        "cursor; later appends never mutate them)."
      ],
      "discriminator": [
        115,
        65,
        62,
        122,
        178,
        145,
        229,
        91
      ],
      "accounts": [
        {
          "name": "asset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "asset.parcel",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "meta",
          "docs": [
            "Per-asset snapshot cursor (created lazily on the first snapshot)."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  115,
                  110,
                  97,
                  112,
                  115,
                  104,
                  111,
                  116,
                  95,
                  109,
                  101,
                  116,
                  97
                ]
              },
              {
                "kind": "account",
                "path": "asset"
              }
            ]
          }
        },
        {
          "name": "snapshot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  115,
                  110,
                  97,
                  112,
                  115,
                  104,
                  111,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "asset"
              },
              {
                "kind": "account",
                "path": "meta.count",
                "account": "spatialSnapshotMeta"
              }
            ]
          }
        },
        {
          "name": "payer",
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
          "name": "label",
          "type": "string"
        }
      ]
    },
    {
      "name": "createStakePool",
      "discriminator": [
        198,
        175,
        88,
        63,
        128,
        43,
        8,
        214
      ],
      "accounts": [
        {
          "name": "regionRegistry",
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
          "name": "stakePool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "payer",
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
          "name": "rewardRateBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "createValidatorRelationshipEdge",
      "discriminator": [
        174,
        195,
        231,
        155,
        189,
        138,
        224,
        229
      ],
      "accounts": [
        {
          "name": "edge",
          "writable": true
        },
        {
          "name": "fromProfile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "fromWallet"
              }
            ]
          }
        },
        {
          "name": "toProfile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "toProfile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "fromWallet",
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
          "name": "edgeType",
          "type": "u8"
        },
        {
          "name": "weightBps",
          "type": "u16"
        }
      ]
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
      "name": "createVerificationTask",
      "discriminator": [
        117,
        19,
        108,
        104,
        139,
        244,
        142,
        109
      ],
      "accounts": [
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              }
            ]
          }
        },
        {
          "name": "requester",
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
          "name": "taskId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "subject",
          "type": "pubkey"
        },
        {
          "name": "taskClass",
          "type": "u8"
        },
        {
          "name": "rewardLamports",
          "type": "u64"
        },
        {
          "name": "deadline",
          "type": "i64"
        },
        {
          "name": "descriptionHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "requiredValidators",
          "type": "u8"
        }
      ]
    },
    {
      "name": "createWorldRegistry",
      "discriminator": [
        124,
        83,
        32,
        9,
        197,
        29,
        115,
        8
      ],
      "accounts": [
        {
          "name": "worldRegistry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  111,
                  114,
                  108,
                  100,
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "declareValidatorCapability",
      "discriminator": [
        13,
        57,
        210,
        102,
        203,
        195,
        88,
        146
      ],
      "accounts": [
        {
          "name": "capability",
          "writable": true
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
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
          "name": "capabilityCode",
          "type": "u8"
        },
        {
          "name": "level",
          "type": "u8"
        },
        {
          "name": "evidenceHash",
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
      "name": "depositEscrow",
      "discriminator": [
        226,
        112,
        158,
        176,
        178,
        118,
        153,
        128
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "escrowVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "escrowRecord"
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
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "buyer",
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
          "name": "depositAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "depositStake",
      "discriminator": [
        160,
        167,
        9,
        220,
        74,
        243,
        228,
        43
      ],
      "accounts": [
        {
          "name": "regionRegistry",
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
          "name": "stakePool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "validatorStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator",
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
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "dismissReport",
      "discriminator": [
        17,
        115,
        109,
        189,
        166,
        8,
        146,
        58
      ],
      "accounts": [
        {
          "name": "slashingReport",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  108,
                  97,
                  115,
                  104,
                  105,
                  110,
                  103,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "slashingReport.reporter",
                "account": "slashingReport"
              },
              {
                "kind": "account",
                "path": "slashingReport.evidenceHash",
                "account": "slashingReport"
              }
            ]
          }
        },
        {
          "name": "stakePool",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "reporter",
          "writable": true
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "disputeEscrow",
      "discriminator": [
        198,
        174,
        139,
        70,
        87,
        79,
        181,
        139
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
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
          "name": "dispute",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  112,
                  117,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "arg",
                "path": "caseHash"
              }
            ]
          }
        },
        {
          "name": "filer",
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
          "name": "caseHash",
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
      "name": "disputeGuardianClaim",
      "discriminator": [
        156,
        186,
        18,
        163,
        25,
        130,
        148,
        101
      ],
      "accounts": [
        {
          "name": "guardianClaim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  117,
                  97,
                  114,
                  100,
                  105,
                  97,
                  110,
                  95,
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "guardianClaim.claim",
                "account": "guardianClaim"
              }
            ]
          }
        },
        {
          "name": "caller",
          "docs": [
            "Must be the original triggerer or registry admin."
          ],
          "signer": true
        },
        {
          "name": "registry",
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
        }
      ],
      "args": []
    },
    {
      "name": "disputeSlashing",
      "discriminator": [
        119,
        42,
        202,
        53,
        178,
        248,
        69,
        63
      ],
      "accounts": [
        {
          "name": "slashingReport",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  108,
                  97,
                  115,
                  104,
                  105,
                  110,
                  103,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "slashingReport.reporter",
                "account": "slashingReport"
              },
              {
                "kind": "account",
                "path": "slashingReport.evidenceHash",
                "account": "slashingReport"
              }
            ]
          }
        },
        {
          "name": "stakePool",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "offender",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "appealReason",
          "type": "string"
        }
      ]
    },
    {
      "name": "distributeRewards",
      "discriminator": [
        97,
        6,
        227,
        255,
        124,
        165,
        3,
        148
      ],
      "accounts": [
        {
          "name": "stakePool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "treasury",
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
          "name": "subject"
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
      "name": "endorseValidatorAdd",
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
                "account": "validatorEndorsement"
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
      "name": "endorseValidatorRemoval",
      "discriminator": [
        88,
        47,
        224,
        49,
        1,
        200,
        113,
        178
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
                "account": "validatorEndorsement"
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
      "name": "executeEmergencyInjection",
      "discriminator": [
        170,
        146,
        57,
        202,
        182,
        243,
        233,
        187
      ],
      "accounts": [
        {
          "name": "emergencyInjection",
          "writable": true
        },
        {
          "name": "registry",
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
          "name": "newValidatorActivity",
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
                  97,
                  99,
                  116,
                  105,
                  118,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "registry"
              },
              {
                "kind": "account",
                "path": "emergencyInjection.candidate",
                "account": "emergencyInjection"
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "executeJudgment",
      "discriminator": [
        78,
        95,
        78,
        183,
        3,
        107,
        42,
        180
      ],
      "accounts": [
        {
          "name": "dispute",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  112,
                  117,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "account",
                "path": "dispute.caseHash",
                "account": "dispute"
              }
            ]
          }
        },
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
          "name": "ownership",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
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
      "name": "expireEscrow",
      "discriminator": [
        49,
        150,
        54,
        201,
        45,
        106,
        39,
        175
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
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
          "name": "caller",
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
      "name": "fileChallenge",
      "discriminator": [
        200,
        165,
        22,
        96,
        219,
        87,
        83,
        30
      ],
      "accounts": [
        {
          "name": "challenge",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  97,
                  108,
                  108,
                  101,
                  110,
                  103,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "claimAccount"
              },
              {
                "kind": "account",
                "path": "challenger"
              }
            ]
          }
        },
        {
          "name": "claimAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claimAccount.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claimAccount.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "challenger",
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
          "name": "challengeHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "requiredVotes",
          "type": "u8"
        }
      ]
    },
    {
      "name": "fileDispute",
      "discriminator": [
        210,
        63,
        221,
        114,
        212,
        97,
        195,
        156
      ],
      "accounts": [
        {
          "name": "dispute",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  112,
                  117,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "arg",
                "path": "caseHash"
              }
            ]
          }
        },
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "filer",
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
          "name": "caseHash",
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
      "name": "fileFraudAppeal",
      "discriminator": [
        223,
        21,
        14,
        161,
        5,
        56,
        84,
        161
      ],
      "accounts": [
        {
          "name": "appeal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  112,
                  112,
                  101,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "restriction"
              },
              {
                "kind": "account",
                "path": "appellant"
              },
              {
                "kind": "arg",
                "path": "nonce"
              }
            ]
          }
        },
        {
          "name": "restriction",
          "writable": true
        },
        {
          "name": "report",
          "writable": true
        },
        {
          "name": "appellant",
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
          "type": "u16"
        },
        {
          "name": "evidenceHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "note",
          "type": "string"
        }
      ]
    },
    {
      "name": "finalizeAppealReview",
      "discriminator": [
        175,
        147,
        250,
        36,
        162,
        172,
        217,
        238
      ],
      "accounts": [
        {
          "name": "review",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  118,
                  105,
                  101,
                  119,
                  95,
                  99,
                  97,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "appeal"
              }
            ]
          }
        },
        {
          "name": "appeal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  112,
                  112,
                  101,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "appeal.restriction",
                "account": "appeal"
              },
              {
                "kind": "account",
                "path": "appeal.appellant",
                "account": "appeal"
              },
              {
                "kind": "account",
                "path": "appeal.nonce",
                "account": "appeal"
              }
            ]
          }
        },
        {
          "name": "restriction",
          "writable": true
        },
        {
          "name": "capability",
          "writable": true
        },
        {
          "name": "report",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  114,
                  97,
                  117,
                  100,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "report.accused",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.reporter",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.nonce",
                "account": "fraudReport"
              }
            ]
          }
        },
        {
          "name": "finalizer",
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
      "name": "finalizeCredential",
      "discriminator": [
        159,
        151,
        12,
        193,
        60,
        47,
        14,
        45
      ],
      "accounts": [
        {
          "name": "credentialRequest",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  101,
                  110,
                  116,
                  105,
                  97,
                  108,
                  95,
                  114,
                  101,
                  113,
                  117,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "credentialRequest.requestHash",
                "account": "credentialRequest"
              }
            ]
          }
        },
        {
          "name": "thresholdCredential",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  104,
                  114,
                  101,
                  115,
                  104,
                  111,
                  108,
                  100,
                  95,
                  99,
                  114,
                  101,
                  100,
                  101,
                  110,
                  116,
                  105,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "credentialRequest.requestHash",
                "account": "credentialRequest"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "payer",
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
      "name": "finalizeFraudReview",
      "discriminator": [
        83,
        178,
        116,
        175,
        100,
        172,
        99,
        75
      ],
      "accounts": [
        {
          "name": "review",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  118,
                  105,
                  101,
                  119,
                  95,
                  99,
                  97,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "report"
              }
            ]
          }
        },
        {
          "name": "report",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  114,
                  97,
                  117,
                  100,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "report.accused",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.reporter",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.nonce",
                "account": "fraudReport"
              }
            ]
          }
        },
        {
          "name": "restriction",
          "writable": true
        },
        {
          "name": "capability",
          "writable": true
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "report.accused",
                "account": "fraudReport"
              }
            ]
          }
        },
        {
          "name": "finalizer",
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
      "name": "finalizeQuorum",
      "discriminator": [
        150,
        75,
        200,
        115,
        28,
        182,
        119,
        71
      ],
      "accounts": [
        {
          "name": "tally",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  113,
                  117,
                  111,
                  114,
                  117,
                  109,
                  95,
                  116,
                  97,
                  108,
                  108,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              }
            ]
          }
        },
        {
          "name": "claim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "freezeParcel",
      "discriminator": [
        245,
        48,
        165,
        183,
        59,
        143,
        136,
        149
      ],
      "accounts": [
        {
          "name": "dispute",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  112,
                  117,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              },
              {
                "kind": "account",
                "path": "dispute.caseHash",
                "account": "dispute"
              }
            ]
          }
        },
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
          "name": "validator",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "fundTaskEscrow",
      "discriminator": [
        129,
        231,
        7,
        54,
        11,
        67,
        185,
        214
      ],
      "accounts": [
        {
          "name": "task"
        },
        {
          "name": "quote",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  115,
                  111,
                  117,
                  114,
                  99,
                  101,
                  95,
                  113,
                  117,
                  111,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "feePolicy",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101,
                  95,
                  112,
                  111,
                  108,
                  105,
                  99,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "taskEscrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "taskEscrow"
              }
            ]
          }
        },
        {
          "name": "treasury",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "requester",
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
      "name": "generateOwnershipRoot",
      "discriminator": [
        111,
        202,
        252,
        133,
        234,
        141,
        84,
        97
      ],
      "accounts": [
        {
          "name": "zoneSet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  122,
                  111,
                  110,
                  101,
                  95,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet.zoneId",
                "account": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "ownershipRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        },
        {
          "name": "instructions",
          "docs": [
            "(must be the Instructions sysvar holding this transaction's",
            "Ed25519 precompile attestation for the new root)."
          ]
        }
      ],
      "args": [
        {
          "name": "newMerkleRoot",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "newSnapshotCid",
          "type": "string"
        },
        {
          "name": "newSnapshotHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "commitmentCount",
          "type": "u32"
        },
        {
          "name": "authoritySignature",
          "type": {
            "array": [
              "u8",
              64
            ]
          }
        }
      ]
    },
    {
      "name": "grantConditionalRight",
      "discriminator": [
        203,
        9,
        162,
        219,
        52,
        15,
        55,
        80
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
          "name": "conditionDeadline",
          "type": "i64"
        },
        {
          "name": "conditionDesc",
          "type": "string"
        },
        {
          "name": "gracePeriodSecs",
          "type": "i64"
        },
        {
          "name": "notes",
          "type": "string"
        }
      ]
    },
    {
      "name": "grantIdentityRight",
      "docs": [
        "Grant an identity-based right on a parcel. Only the current holder",
        "of the canonical ownership right may sign. The `identity` must be a",
        "valid Identity PDA.",
        "",
        "Identity is a separate binding/provenance layer: this creates the",
        "optional identity↔parcel link (`IdentityRights`), it is never the",
        "ownership record itself (ownership lives at `[\"ownership\", parcel]`).",
        "",
        "PDA: `[\"identity_rights\", identity, parcel, rights_kind]`."
      ],
      "discriminator": [
        36,
        242,
        147,
        162,
        0,
        217,
        216,
        72
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "identity"
        },
        {
          "name": "identityRights",
          "writable": true
        },
        {
          "name": "granter",
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
          "name": "rightsKind",
          "type": "u8"
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
      "name": "grantRight",
      "docs": [
        "Grant a non-ownership right on a parcel to `holder`. Holder-only.",
        "",
        "`nonce` must equal the parcel's current `rights_count`, which is",
        "incremented so every right gets a unique PDA. OWNERSHIP is excluded",
        "here — it lives solely at the canonical `[\"ownership\", parcel]` PDA",
        "and only moves via `transfer_parcel`."
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
      "name": "heartbeat",
      "discriminator": [
        202,
        104,
        56,
        6,
        240,
        170,
        63,
        134
      ],
      "accounts": [
        {
          "name": "activityTracker",
          "writable": true
        },
        {
          "name": "registry",
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
          "name": "validator",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "initSpatialAsset",
      "discriminator": [
        202,
        140,
        58,
        139,
        229,
        206,
        0,
        84
      ],
      "accounts": [
        {
          "name": "asset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "parcel",
          "docs": [
            "The parcel to extend (self-pinned: its own PDA seeds)."
          ],
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
          "name": "registrar",
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
          "name": "dimensionality",
          "type": "u8"
        },
        {
          "name": "elevationMinMm",
          "type": "i32"
        },
        {
          "name": "elevationMaxMm",
          "type": "i32"
        }
      ]
    },
    {
      "name": "initValidatorProfile",
      "discriminator": [
        135,
        94,
        41,
        156,
        111,
        202,
        31,
        61
      ],
      "accounts": [
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
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
          "name": "note",
          "type": "string"
        }
      ]
    },
    {
      "name": "initializeValidatorReputation",
      "discriminator": [
        199,
        93,
        127,
        85,
        145,
        10,
        252,
        73
      ],
      "accounts": [
        {
          "name": "reputation",
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
                  114,
                  101,
                  112,
                  117,
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
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator"
        },
        {
          "name": "registry",
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
          "name": "payer",
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
      "name": "initiateUnbonding",
      "discriminator": [
        75,
        33,
        161,
        251,
        137,
        112,
        71,
        37
      ],
      "accounts": [
        {
          "name": "validatorStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "stakePool",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "validator",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "invalidateProof",
      "discriminator": [
        122,
        166,
        31,
        119,
        225,
        47,
        26,
        193
      ],
      "accounts": [
        {
          "name": "zoneSet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  122,
                  111,
                  110,
                  101,
                  95,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet.zoneId",
                "account": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "staleVersion",
          "type": "u32"
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
        "F4: every declared validator must also be a member of the on-chain",
        "`ValidatorRegistry` — a self-declared key set can never form a quorum.",
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
          "name": "ownership",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "registry",
          "docs": [
            "On-chain validator registry (F4): every declared forfeit validator",
            "must be a member of `registry.validators`."
          ],
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
      "name": "linkCrossBorderToSession",
      "discriminator": [
        237,
        10,
        150,
        195,
        138,
        70,
        113,
        25
      ],
      "accounts": [
        {
          "name": "crossBorderVerification",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "binding"
              }
            ]
          }
        },
        {
          "name": "binding"
        },
        {
          "name": "claim",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "session",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  115,
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
                "path": "session.claim",
                "account": "verificationSession"
              },
              {
                "kind": "account",
                "path": "session.sessionId",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "jurisdiction"
        },
        {
          "name": "caller",
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
      "name": "migrateRights",
      "discriminator": [
        81,
        152,
        160,
        134,
        189,
        248,
        189,
        215
      ],
      "accounts": [
        {
          "name": "oldParcel",
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
                "path": "oldParcel.id",
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "oldParcel"
              }
            ]
          }
        },
        {
          "name": "newParcel",
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
                "path": "newParcel.id",
                "account": "parcel"
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
      "args": []
    },
    {
      "name": "mutualCancelEscrow",
      "discriminator": [
        241,
        12,
        37,
        171,
        212,
        236,
        201,
        151
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "escrowVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "escrowRecord"
              }
            ]
          }
        },
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
          "name": "seller",
          "writable": true
        },
        {
          "name": "buyer",
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "nominateValidator",
      "discriminator": [
        54,
        117,
        124,
        157,
        36,
        153,
        48,
        128
      ],
      "accounts": [
        {
          "name": "nomination",
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
                  110,
                  111,
                  109,
                  105,
                  110,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "registry"
              },
              {
                "kind": "arg",
                "path": "candidate"
              }
            ]
          }
        },
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "sponsor",
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
          "name": "candidate",
          "type": "pubkey"
        },
        {
          "name": "documentsHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "locationHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "countryCode",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        },
        {
          "name": "recentBlockhash",
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
      "name": "openAppealReview",
      "discriminator": [
        84,
        185,
        145,
        44,
        10,
        217,
        140,
        69
      ],
      "accounts": [
        {
          "name": "appeal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  112,
                  112,
                  101,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "appeal.restriction",
                "account": "appeal"
              },
              {
                "kind": "account",
                "path": "appeal.appellant",
                "account": "appeal"
              },
              {
                "kind": "account",
                "path": "appeal.nonce",
                "account": "appeal"
              }
            ]
          }
        },
        {
          "name": "restriction"
        },
        {
          "name": "review",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  118,
                  105,
                  101,
                  119,
                  95,
                  99,
                  97,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "appeal"
              }
            ]
          }
        },
        {
          "name": "registry",
          "docs": [
            "On-chain validator registry (F3): committee pool members must be",
            "registered validators."
          ],
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
          "name": "appellant",
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
      "name": "openFraudReview",
      "discriminator": [
        111,
        26,
        122,
        231,
        162,
        127,
        252,
        180
      ],
      "accounts": [
        {
          "name": "report",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  114,
                  97,
                  117,
                  100,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "report.accused",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.reporter",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.nonce",
                "account": "fraudReport"
              }
            ]
          }
        },
        {
          "name": "review",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  118,
                  105,
                  101,
                  119,
                  95,
                  99,
                  97,
                  115,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "report"
              }
            ]
          }
        },
        {
          "name": "registry",
          "docs": [
            "On-chain validator registry (F3): committee pool members must be",
            "registered validators."
          ],
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
          "name": "reporter",
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
      "name": "openVerificationSession",
      "discriminator": [
        233,
        132,
        252,
        194,
        21,
        120,
        250,
        249
      ],
      "accounts": [
        {
          "name": "session",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  115,
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
                "path": "claim"
              },
              {
                "kind": "arg",
                "path": "sessionId"
              }
            ]
          }
        },
        {
          "name": "claim",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "sessionTracker",
          "docs": [
            "Singleton guard: enforces single active session per claim."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109,
                  95,
                  115,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110,
                  95,
                  116,
                  114,
                  97,
                  99,
                  107,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              }
            ]
          }
        },
        {
          "name": "opener",
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
          "name": "sessionId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "parcelType",
          "type": "u8"
        },
        {
          "name": "region",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        }
      ]
    },
    {
      "name": "pauseProgram",
      "discriminator": [
        91,
        86,
        253,
        175,
        66,
        236,
        172,
        124
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "admin",
          "signer": true
        }
      ],
      "args": []
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
      "name": "proposeValidator",
      "discriminator": [
        177,
        134,
        171,
        144,
        175,
        45,
        190,
        62
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
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
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator"
        },
        {
          "name": "proposer",
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
          "name": "validator",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "proposeValidatorRemoval",
      "discriminator": [
        217,
        106,
        65,
        124,
        51,
        78,
        35,
        190
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
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
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator"
        },
        {
          "name": "proposer",
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
          "name": "validator",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "queueEmergencyInjection",
      "discriminator": [
        153,
        11,
        101,
        155,
        141,
        4,
        25,
        86
      ],
      "accounts": [
        {
          "name": "emergencyInjection",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  109,
                  101,
                  114,
                  103,
                  101,
                  110,
                  99,
                  121,
                  95,
                  105,
                  110,
                  106,
                  101,
                  99,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "registry"
              },
              {
                "kind": "arg",
                "path": "candidate"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "candidate",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "quoteTaskResources",
      "discriminator": [
        185,
        225,
        105,
        50,
        115,
        131,
        246,
        66
      ],
      "accounts": [
        {
          "name": "task"
        },
        {
          "name": "quote",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  115,
                  111,
                  117,
                  114,
                  99,
                  101,
                  95,
                  113,
                  117,
                  111,
                  116,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "quoter",
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
          "name": "resourceCost",
          "type": "u64"
        }
      ]
    },
    {
      "name": "reactivateObserver",
      "discriminator": [
        31,
        227,
        195,
        57,
        233,
        15,
        184,
        50
      ],
      "accounts": [
        {
          "name": "observer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  98,
                  115,
                  101,
                  114,
                  118,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "observer.wallet",
                "account": "observer"
              }
            ]
          }
        },
        {
          "name": "authority"
        },
        {
          "name": "registry",
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
        }
      ],
      "args": []
    },
    {
      "name": "rebindCrossBorderIdentity",
      "discriminator": [
        144,
        28,
        210,
        61,
        36,
        94,
        44,
        59
      ],
      "accounts": [
        {
          "name": "oldBinding",
          "writable": true
        },
        {
          "name": "newBinding",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
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
                "path": "jurisdiction"
              },
              {
                "kind": "account",
                "path": "oldBinding.identityHash",
                "account": "jurisdictionBinding"
              }
            ]
          }
        },
        {
          "name": "identity"
        },
        {
          "name": "jurisdiction",
          "writable": true
        },
        {
          "name": "prover",
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
          "name": "credentialCommitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "proofData",
          "type": "bytes"
        },
        {
          "name": "nullifierNonce",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "expiresAt",
          "type": "i64"
        }
      ]
    },
    {
      "name": "recordAttestationOutcome",
      "discriminator": [
        61,
        154,
        146,
        140,
        172,
        155,
        47,
        223
      ],
      "accounts": [
        {
          "name": "reputation",
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
                  114,
                  101,
                  112,
                  117,
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
                "path": "reputation.validator",
                "account": "validatorReputation"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "confirmed",
          "type": "bool"
        }
      ]
    },
    {
      "name": "recordAuditEntry",
      "discriminator": [
        187,
        75,
        4,
        88,
        143,
        90,
        203,
        144
      ],
      "accounts": [
        {
          "name": "auditEntry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  117,
                  100,
                  105,
                  116,
                  95,
                  101,
                  110,
                  116,
                  114,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "entity"
              },
              {
                "kind": "arg",
                "path": "sequence"
              }
            ]
          }
        },
        {
          "name": "entity",
          "docs": [
            "Must be an account owned by this program so arbitrary keys cannot",
            "pollute the audit log (M-2)."
          ]
        },
        {
          "name": "actor",
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
          "name": "sequence",
          "type": "u32"
        },
        {
          "name": "action",
          "type": "u8"
        },
        {
          "name": "fromStatus",
          "type": "u8"
        },
        {
          "name": "toStatus",
          "type": "u8"
        },
        {
          "name": "metadataHash",
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
      "name": "recordCrossBorderVerification",
      "discriminator": [
        243,
        167,
        81,
        52,
        73,
        2,
        176,
        244
      ],
      "accounts": [
        {
          "name": "verification",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              },
              {
                "kind": "arg",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "task",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              }
            ]
          }
        },
        {
          "name": "requirement"
        },
        {
          "name": "assignment",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  97,
                  115,
                  115,
                  105,
                  103,
                  110,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              },
              {
                "kind": "arg",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "arg",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "binding"
        },
        {
          "name": "payer",
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
          "name": "taskId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "reqIndex",
          "type": "u8"
        },
        {
          "name": "validator",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "recordSessionAttestation",
      "discriminator": [
        210,
        151,
        181,
        21,
        25,
        112,
        157,
        51
      ],
      "accounts": [
        {
          "name": "session",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  115,
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
                "path": "session.claim",
                "account": "verificationSession"
              },
              {
                "kind": "account",
                "path": "session.sessionId",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "sessionTracker",
          "docs": [
            "Must match the tracker for this claim (cleared when quorum reached)."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109,
                  95,
                  115,
                  101,
                  115,
                  115,
                  105,
                  111,
                  110,
                  95,
                  116,
                  114,
                  97,
                  99,
                  107,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "session.claim",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "signer",
          "docs": [
            "The session opener, registry admin, or a registered validator must sign."
          ],
          "signer": true
        }
      ],
      "args": [
        {
          "name": "isConfirmatory",
          "type": "bool"
        }
      ]
    },
    {
      "name": "recordSessionEvidence",
      "discriminator": [
        251,
        34,
        251,
        2,
        160,
        243,
        29,
        142
      ],
      "accounts": [
        {
          "name": "session",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  115,
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
                "path": "session.claim",
                "account": "verificationSession"
              },
              {
                "kind": "account",
                "path": "session.sessionId",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "signer",
          "docs": [
            "The session opener, registry admin, or a registered validator must sign."
          ],
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "recordSessionObservation",
      "discriminator": [
        90,
        81,
        4,
        66,
        160,
        144,
        79,
        197
      ],
      "accounts": [
        {
          "name": "session",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  115,
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
                "path": "session.claim",
                "account": "verificationSession"
              },
              {
                "kind": "account",
                "path": "session.sessionId",
                "account": "verificationSession"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "signer",
          "docs": [
            "The session opener, registry admin, or a registered validator must sign."
          ],
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "refundTaskEscrow",
      "discriminator": [
        67,
        95,
        34,
        100,
        202,
        45,
        223,
        197
      ],
      "accounts": [
        {
          "name": "task"
        },
        {
          "name": "taskEscrow",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "taskEscrow"
              }
            ]
          }
        },
        {
          "name": "requester",
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
      "name": "register3dModel",
      "docs": [
        "Register an off-chain 3D model claim against an anchored geometry",
        "version (RFC-012 §5 `ThreeDModel`, Design Rule 10 — never on-chain",
        "geometry, only hash + storage pointer)."
      ],
      "discriminator": [
        210,
        186,
        7,
        195,
        234,
        47,
        114,
        9
      ],
      "accounts": [
        {
          "name": "asset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "asset.parcel",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "meta",
          "docs": [
            "Per-asset model cursor (created lazily on first registration)."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  104,
                  114,
                  101,
                  101,
                  95,
                  100,
                  95,
                  109,
                  111,
                  100,
                  101,
                  108,
                  95,
                  109,
                  101,
                  116,
                  97
                ]
              },
              {
                "kind": "account",
                "path": "asset"
              }
            ]
          }
        },
        {
          "name": "model",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  104,
                  114,
                  101,
                  101,
                  95,
                  100,
                  95,
                  109,
                  111,
                  100,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "asset"
              },
              {
                "kind": "account",
                "path": "meta.count",
                "account": "threeDModelMeta"
              }
            ]
          }
        },
        {
          "name": "payer",
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
          "name": "modelHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "storageReference",
          "type": "string"
        },
        {
          "name": "modelFormat",
          "type": "u8"
        },
        {
          "name": "geometryVersion",
          "type": "u32"
        }
      ]
    },
    {
      "name": "registerDevice",
      "discriminator": [
        210,
        151,
        56,
        68,
        22,
        158,
        90,
        193
      ],
      "accounts": [
        {
          "name": "device",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  118,
                  105,
                  99,
                  101,
                  95,
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
                "path": "owner"
              },
              {
                "kind": "arg",
                "path": "deviceNonce"
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
          "name": "deviceNonce",
          "type": "u16"
        },
        {
          "name": "deviceKey",
          "type": "pubkey"
        },
        {
          "name": "source",
          "type": "u8"
        },
        {
          "name": "capabilities",
          "type": "u64"
        },
        {
          "name": "metadataRef",
          "type": "string"
        }
      ]
    },
    {
      "name": "registerJurisdiction",
      "discriminator": [
        6,
        255,
        84,
        162,
        130,
        198,
        135,
        7
      ],
      "accounts": [
        {
          "name": "jurisdiction",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  106,
                  117,
                  114,
                  105,
                  115,
                  100,
                  105,
                  99,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "arg",
                "path": "countryCode"
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
          "name": "registry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "countryCode",
          "type": {
            "array": [
              "u8",
              16
            ]
          }
        },
        {
          "name": "jurisdictionName",
          "type": "string"
        },
        {
          "name": "credentialSchemaCid",
          "type": "string"
        },
        {
          "name": "revocationRegistry",
          "type": "pubkey"
        },
        {
          "name": "verificationKeyHash",
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
        }
      ]
    },
    {
      "name": "registerObserver",
      "discriminator": [
        95,
        238,
        80,
        77,
        247,
        96,
        2,
        225
      ],
      "accounts": [
        {
          "name": "observer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  98,
                  115,
                  101,
                  114,
                  118,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
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
          "name": "identity",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "registerParcel",
      "docs": [
        "Register a new parcel on-chain. The signer receives the canonical",
        "ownership right (`Rights` PDA at `[\"ownership\", parcel]`), which is",
        "the single source of protocol-level control over the parcel.",
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
          "name": "ownership",
          "docs": [
            "The canonical ownership right (RRR: this — not the parcel — is the",
            "ownership record). Created in the same instruction as the parcel."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
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
      "name": "registerZoneSet",
      "discriminator": [
        161,
        121,
        72,
        36,
        152,
        206,
        45,
        172
      ],
      "accounts": [
        {
          "name": "registry",
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
          "name": "zoneId"
        },
        {
          "name": "zoneSet",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  122,
                  111,
                  110,
                  101,
                  95,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneId"
              }
            ]
          }
        },
        {
          "name": "ownershipRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet"
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
          "name": "snapshotCid",
          "type": "string"
        },
        {
          "name": "snapshotHash",
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
      "name": "rehabilitateRestriction",
      "discriminator": [
        161,
        98,
        41,
        226,
        120,
        112,
        91,
        107
      ],
      "accounts": [
        {
          "name": "restriction",
          "writable": true
        },
        {
          "name": "report",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  114,
                  97,
                  117,
                  100,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "report.accused",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.reporter",
                "account": "fraudReport"
              },
              {
                "kind": "account",
                "path": "report.nonce",
                "account": "fraudReport"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "capability",
          "writable": true
        },
        {
          "name": "wallet",
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
      "name": "removeValidatorFromRegistry",
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
          "name": "adminSigner",
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
                "kind": "account",
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
      "name": "renewRight",
      "discriminator": [
        244,
        248,
        169,
        165,
        181,
        112,
        121,
        237
      ],
      "accounts": [
        {
          "name": "rights",
          "writable": true
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
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "holder",
          "docs": [
            "Holder must sign."
          ],
          "signer": true
        },
        {
          "name": "granter",
          "docs": [
            "Granter must co-sign (original granter or current ownership holder)."
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
          "name": "nonce",
          "type": "u8"
        },
        {
          "name": "newExpiresAt",
          "type": "i64"
        },
        {
          "name": "newNotes",
          "type": "string"
        }
      ]
    },
    {
      "name": "reportEquivocation",
      "discriminator": [
        140,
        250,
        136,
        199,
        9,
        49,
        41,
        16
      ],
      "accounts": [
        {
          "name": "stakePool",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "offenderStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "offenderStake.validator",
                "account": "validatorStake"
              }
            ]
          }
        },
        {
          "name": "slashingReport",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  108,
                  97,
                  115,
                  104,
                  105,
                  110,
                  103,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "reporter"
              },
              {
                "kind": "arg",
                "path": "evidenceHash"
              }
            ]
          }
        },
        {
          "name": "reporter",
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
          "name": "evidenceHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "offenseDetails",
          "type": {
            "array": [
              "u8",
              64
            ]
          }
        }
      ]
    },
    {
      "name": "reportValidatorOffense",
      "discriminator": [
        26,
        28,
        132,
        175,
        77,
        1,
        72,
        66
      ],
      "accounts": [
        {
          "name": "stakePool",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "offenderStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "offenderStake.validator",
                "account": "validatorStake"
              }
            ]
          }
        },
        {
          "name": "slashingReport",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  108,
                  97,
                  115,
                  104,
                  105,
                  110,
                  103,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "reporter"
              },
              {
                "kind": "arg",
                "path": "evidenceHash"
              }
            ]
          }
        },
        {
          "name": "reporter",
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
          "name": "offenseKind",
          "type": "u8"
        },
        {
          "name": "evidenceHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "offenseDetails",
          "type": {
            "array": [
              "u8",
              64
            ]
          }
        }
      ]
    },
    {
      "name": "requestCredential",
      "discriminator": [
        250,
        55,
        225,
        61,
        98,
        70,
        116,
        139
      ],
      "accounts": [
        {
          "name": "credentialRequest",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  101,
                  110,
                  116,
                  105,
                  97,
                  108,
                  95,
                  114,
                  101,
                  113,
                  117,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "requestHash"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "regionRegistry"
        },
        {
          "name": "prover",
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
          "name": "requestHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "purpose",
          "type": "string"
        },
        {
          "name": "disclosureType",
          "type": "u8"
        }
      ]
    },
    {
      "name": "requestGenesis",
      "discriminator": [
        21,
        184,
        230,
        139,
        66,
        62,
        207,
        74
      ],
      "accounts": [
        {
          "name": "genesisRequest",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  101,
                  110,
                  101,
                  115,
                  105,
                  115,
                  95,
                  114,
                  101,
                  113,
                  117,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "countryCode"
              }
            ]
          }
        },
        {
          "name": "worldRegistry",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  111,
                  114,
                  108,
                  100,
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
          "name": "requester",
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
          "name": "countryCode",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        }
      ]
    },
    {
      "name": "resolveGuardianClaim",
      "discriminator": [
        125,
        182,
        113,
        121,
        218,
        93,
        77,
        92
      ],
      "accounts": [
        {
          "name": "guardianClaim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  117,
                  97,
                  114,
                  100,
                  105,
                  97,
                  110,
                  95,
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "guardianClaim.claim",
                "account": "guardianClaim"
              }
            ]
          }
        },
        {
          "name": "caller",
          "docs": [
            "Must be the original triggerer or registry admin."
          ],
          "signer": true
        },
        {
          "name": "registry",
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
        }
      ],
      "args": []
    },
    {
      "name": "revokeCrossBorder",
      "discriminator": [
        32,
        235,
        100,
        117,
        164,
        172,
        185,
        139
      ],
      "accounts": [
        {
          "name": "crossBorderVerification",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "crossBorderVerification.binding",
                "account": "crossBorderVerification"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority"
        }
      ],
      "args": []
    },
    {
      "name": "revokeIdentityRight",
      "docs": [
        "Revoke an identity-based right. The original granter may revoke."
      ],
      "discriminator": [
        201,
        116,
        44,
        133,
        73,
        8,
        192,
        141
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
          "name": "identityRights",
          "writable": true
        },
        {
          "name": "identity"
        },
        {
          "name": "granter",
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
          "name": "rightsKind",
          "type": "u8"
        }
      ]
    },
    {
      "name": "revokeJurisdictionalIdentity",
      "discriminator": [
        196,
        187,
        133,
        202,
        68,
        24,
        149,
        40
      ],
      "accounts": [
        {
          "name": "binding",
          "writable": true
        },
        {
          "name": "jurisdiction"
        },
        {
          "name": "identity"
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "reason",
          "type": "string"
        }
      ]
    },
    {
      "name": "revokeRight",
      "docs": [
        "Revoke a previously granted (non-ownership) right. The current parcel",
        "holder or the original granter may revoke. The account is closed and",
        "its lamports returned. The parcel's rights counter is decremented so",
        "the freed nonce can be reused by a future grant (safe: the old account",
        "no longer exists). The canonical ownership right is not revocable —",
        "it only moves via `transfer_parcel`."
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
      "name": "rotateDeviceKey",
      "discriminator": [
        96,
        133,
        177,
        36,
        92,
        46,
        59,
        153
      ],
      "accounts": [
        {
          "name": "device",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  118,
                  105,
                  99,
                  101,
                  95,
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
                "path": "device.owner",
                "account": "deviceIdentity"
              },
              {
                "kind": "account",
                "path": "device.deviceNonce",
                "account": "deviceIdentity"
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
          "name": "newDeviceKey",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "routeTask",
      "discriminator": [
        10,
        69,
        178,
        125,
        217,
        194,
        194,
        25
      ],
      "accounts": [
        {
          "name": "assignment",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  97,
                  115,
                  115,
                  105,
                  103,
                  110,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              },
              {
                "kind": "arg",
                "path": "chosen"
              }
            ]
          }
        },
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              }
            ]
          }
        },
        {
          "name": "requirement"
        },
        {
          "name": "chosen",
          "docs": [
            "handler against candidate PDAs in remaining_accounts."
          ]
        },
        {
          "name": "requester",
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
          "name": "taskId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "reqIndex",
          "type": "u8"
        },
        {
          "name": "candidates",
          "type": {
            "vec": "pubkey"
          }
        },
        {
          "name": "chosen",
          "type": "pubkey"
        },
        {
          "name": "competitorCount",
          "type": "u16"
        }
      ]
    },
    {
      "name": "setCoverageIncentive",
      "discriminator": [
        240,
        111,
        147,
        207,
        54,
        63,
        188,
        13
      ],
      "accounts": [
        {
          "name": "coverageIncentive",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  118,
                  101,
                  114,
                  97,
                  103,
                  101,
                  95,
                  105,
                  110,
                  99,
                  101,
                  110,
                  116,
                  105,
                  118,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "demandBps",
          "type": "u16"
        },
        {
          "name": "deficitBps",
          "type": "u16"
        },
        {
          "name": "difficultyBps",
          "type": "u16"
        },
        {
          "name": "strategicBps",
          "type": "u16"
        },
        {
          "name": "maxSubsidyBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "setCrossBorderBindingStatus",
      "discriminator": [
        192,
        154,
        102,
        0,
        127,
        23,
        171,
        206
      ],
      "accounts": [
        {
          "name": "binding",
          "writable": true
        },
        {
          "name": "jurisdictionMin"
        },
        {
          "name": "jurisdictionMax"
        },
        {
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "status",
          "type": "u8"
        }
      ]
    },
    {
      "name": "setDeviceCalibration",
      "discriminator": [
        117,
        26,
        56,
        40,
        77,
        121,
        194,
        213
      ],
      "accounts": [
        {
          "name": "device",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  118,
                  105,
                  99,
                  101,
                  95,
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
                "path": "device.owner",
                "account": "deviceIdentity"
              },
              {
                "kind": "account",
                "path": "device.deviceNonce",
                "account": "deviceIdentity"
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
          "name": "calibrationHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "calibratedAt",
          "type": "i64"
        }
      ]
    },
    {
      "name": "setDeviceStatus",
      "discriminator": [
        151,
        78,
        75,
        125,
        26,
        121,
        118,
        57
      ],
      "accounts": [
        {
          "name": "device",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  118,
                  105,
                  99,
                  101,
                  95,
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
                "path": "device.owner",
                "account": "deviceIdentity"
              },
              {
                "kind": "account",
                "path": "device.deviceNonce",
                "account": "deviceIdentity"
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
    },
    {
      "name": "setFeePolicy",
      "discriminator": [
        91,
        32,
        224,
        109,
        147,
        98,
        71,
        132
      ],
      "accounts": [
        {
          "name": "feePolicy",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101,
                  95,
                  112,
                  111,
                  108,
                  105,
                  99,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "feeBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "setQuorumConfig",
      "discriminator": [
        10,
        126,
        231,
        106,
        111,
        196,
        155,
        199
      ],
      "accounts": [
        {
          "name": "config",
          "writable": true
        },
        {
          "name": "registry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "parcelType",
          "type": "u8"
        },
        {
          "name": "region",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        },
        {
          "name": "requiredAttestations",
          "type": "u8"
        },
        {
          "name": "requiredConfidence",
          "type": "u8"
        }
      ]
    },
    {
      "name": "setValidatorActive",
      "discriminator": [
        211,
        189,
        132,
        233,
        203,
        244,
        186,
        199
      ],
      "accounts": [
        {
          "name": "activityTracker",
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
                  97,
                  99,
                  116,
                  105,
                  118,
                  105,
                  116,
                  121
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
          "name": "registry",
          "writable": true
        },
        {
          "name": "admin",
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
          "name": "validator",
          "type": "pubkey"
        },
        {
          "name": "isActive",
          "type": "bool"
        }
      ]
    },
    {
      "name": "setValidatorAvailability",
      "discriminator": [
        136,
        107,
        121,
        13,
        81,
        250,
        155,
        108
      ],
      "accounts": [
        {
          "name": "availability",
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
                  97,
                  118,
                  97,
                  105,
                  108,
                  97,
                  98,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "profile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
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
          "name": "status",
          "type": "u8"
        }
      ]
    },
    {
      "name": "setValidatorJurisdiction",
      "docs": [
        "Declare / update the validator's operating jurisdiction (RFC-012 Phase 10)."
      ],
      "discriminator": [
        112,
        18,
        100,
        16,
        206,
        107,
        219,
        237
      ],
      "accounts": [
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "jurisdiction",
          "type": {
            "array": [
              "u8",
              2
            ]
          }
        }
      ]
    },
    {
      "name": "setValidatorPresence",
      "discriminator": [
        201,
        97,
        31,
        245,
        58,
        238,
        77,
        146
      ],
      "accounts": [
        {
          "name": "presence",
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
                  112,
                  114,
                  101,
                  115,
                  101,
                  110,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "profile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
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
          "name": "latitudeE7",
          "type": "i32"
        },
        {
          "name": "longitudeE7",
          "type": "i32"
        },
        {
          "name": "accuracyM",
          "type": "u16"
        },
        {
          "name": "provenance",
          "type": "u8"
        },
        {
          "name": "confidenceBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "setValidatorProfileTier",
      "discriminator": [
        95,
        3,
        115,
        155,
        232,
        27,
        37,
        195
      ],
      "accounts": [
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "profile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "tier",
          "type": "u8"
        }
      ]
    },
    {
      "name": "settleEscrow",
      "discriminator": [
        22,
        135,
        160,
        194,
        23,
        186,
        124,
        110
      ],
      "accounts": [
        {
          "name": "escrowRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "escrowVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "escrowRecord"
              }
            ]
          }
        },
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
          "name": "ownership",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "seller",
          "docs": [
            "Seller receives SOL."
          ],
          "writable": true
        },
        {
          "name": "buyer",
          "docs": [
            "Buyer receives excess deposit."
          ],
          "writable": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "signCredential",
      "discriminator": [
        43,
        172,
        154,
        246,
        116,
        16,
        67,
        118
      ],
      "accounts": [
        {
          "name": "credentialRequest",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  101,
                  110,
                  116,
                  105,
                  97,
                  108,
                  95,
                  114,
                  101,
                  113,
                  117,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "credentialRequest.requestHash",
                "account": "credentialRequest"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "validatorSigner",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "slashValidator",
      "discriminator": [
        238,
        57,
        244,
        40,
        132,
        82,
        78,
        5
      ],
      "accounts": [
        {
          "name": "reputation",
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
                  114,
                  101,
                  112,
                  117,
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
                "path": "reputation.validator",
                "account": "validatorReputation"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "reputationPenalty",
          "type": "u16"
        }
      ]
    },
    {
      "name": "subdivideParcel",
      "discriminator": [
        179,
        231,
        244,
        173,
        88,
        234,
        211,
        172
      ],
      "accounts": [
        {
          "name": "originalParcel",
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
                "path": "originalParcel.id",
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "ownership",
          "docs": [
            "Ownership right of the original parcel (auth source for subdivision)."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "originalParcel"
              }
            ]
          }
        },
        {
          "name": "subParcel",
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
                "path": "newId"
              }
            ]
          }
        },
        {
          "name": "subOwnership",
          "docs": [
            "The sub-parcel's canonical ownership right — inherited from the",
            "original parcel's holder in the same instruction."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "subParcel"
              }
            ]
          }
        },
        {
          "name": "subdivisionRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  117,
                  98,
                  100,
                  105,
                  118,
                  105,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "originalParcel"
              },
              {
                "kind": "account",
                "path": "subParcel"
              }
            ]
          }
        },
        {
          "name": "claim",
          "docs": [
            "Verified surveyor claim for the original parcel: the surveyor flow",
            "(`create_claim(SUBDIVISION) → evidence → observations → verify_claim`)",
            "replaces the deprecated parcel-`Attestation` gate."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "originalParcel"
              },
              {
                "kind": "arg",
                "path": "claimId"
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
          "name": "newId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "newName",
          "type": "string"
        },
        {
          "name": "newGeometryHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "claimId",
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
      "name": "submitEvidenceManifest",
      "discriminator": [
        232,
        67,
        67,
        96,
        211,
        196,
        129,
        102
      ],
      "accounts": [
        {
          "name": "manifest",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  105,
                  100,
                  101,
                  110,
                  99,
                  101,
                  95,
                  109,
                  97,
                  110,
                  105,
                  102,
                  101,
                  115,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              },
              {
                "kind": "account",
                "path": "submitter"
              },
              {
                "kind": "arg",
                "path": "nonce"
              }
            ]
          }
        },
        {
          "name": "task",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              }
            ]
          }
        },
        {
          "name": "submitter",
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
          "name": "taskId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "nonce",
          "type": "u16"
        },
        {
          "name": "observation",
          "type": "pubkey"
        },
        {
          "name": "rootHash",
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
      "name": "submitFraudReport",
      "discriminator": [
        31,
        239,
        168,
        126,
        140,
        132,
        55,
        45
      ],
      "accounts": [
        {
          "name": "report",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  114,
                  97,
                  117,
                  100,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "accused"
              },
              {
                "kind": "account",
                "path": "reporter"
              },
              {
                "kind": "arg",
                "path": "nonce"
              }
            ]
          }
        },
        {
          "name": "accused"
        },
        {
          "name": "reporter",
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
          "type": "u16"
        },
        {
          "name": "evidenceHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "reasonCode",
          "type": "u8"
        },
        {
          "name": "note",
          "type": "string"
        },
        {
          "name": "capabilityCode",
          "type": "u8"
        }
      ]
    },
    {
      "name": "submitObservation",
      "discriminator": [
        109,
        4,
        22,
        163,
        138,
        215,
        205,
        180
      ],
      "accounts": [
        {
          "name": "observation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  98,
                  115,
                  101,
                  114,
                  118,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "claim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "validator",
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
          "name": "location",
          "type": {
            "array": [
              "i64",
              2
            ]
          }
        },
        {
          "name": "method",
          "type": "u8"
        },
        {
          "name": "findingsHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "confidence",
          "type": "u8"
        },
        {
          "name": "signatureHash",
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
      "name": "submitObservationV2",
      "discriminator": [
        255,
        214,
        31,
        189,
        240,
        91,
        185,
        152
      ],
      "accounts": [
        {
          "name": "observation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  98,
                  115,
                  101,
                  114,
                  118,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
                  118,
                  50
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              },
              {
                "kind": "account",
                "path": "observer"
              },
              {
                "kind": "arg",
                "path": "nonce"
              }
            ]
          }
        },
        {
          "name": "task",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "arg",
                "path": "taskId"
              }
            ]
          }
        },
        {
          "name": "observer",
          "docs": [
            "fields on ObservationV2 capture subject/capture_device separation."
          ],
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
          "name": "taskId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "nonce",
          "type": "u16"
        },
        {
          "name": "subject",
          "type": "pubkey"
        },
        {
          "name": "captureDevice",
          "type": "pubkey"
        },
        {
          "name": "source",
          "type": "u8"
        },
        {
          "name": "provenance",
          "type": "u8"
        },
        {
          "name": "location",
          "type": {
            "array": [
              "i64",
              2
            ]
          }
        },
        {
          "name": "observedAt",
          "type": "i64"
        },
        {
          "name": "findingsHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "evidenceHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "confidence",
          "type": "u8"
        },
        {
          "name": "signatureHash",
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
      "name": "submitTaskResult",
      "discriminator": [
        39,
        108,
        74,
        4,
        66,
        125,
        157,
        7
      ],
      "accounts": [
        {
          "name": "assignment",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107,
                  95,
                  97,
                  115,
                  115,
                  105,
                  103,
                  110,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              },
              {
                "kind": "account",
                "path": "assignment.validator",
                "account": "taskAssignment"
              }
            ]
          }
        },
        {
          "name": "task",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  97,
                  115,
                  107
                ]
              },
              {
                "kind": "account",
                "path": "task.taskId",
                "account": "verificationTask"
              }
            ]
          }
        },
        {
          "name": "validator",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "taskId",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "outcome",
          "type": "u8"
        },
        {
          "name": "resultHash",
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
      "name": "submitVerificationAttestation",
      "discriminator": [
        104,
        215,
        200,
        186,
        150,
        152,
        120,
        71
      ],
      "accounts": [
        {
          "name": "attestation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110,
                  95,
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
                "path": "claim"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "claim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "observation",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  98,
                  115,
                  101,
                  114,
                  118,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "claim"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator",
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
          "name": "result",
          "type": "u8"
        },
        {
          "name": "confidence",
          "type": "u8"
        },
        {
          "name": "signatureHash",
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
      "name": "suspendObserver",
      "discriminator": [
        187,
        179,
        16,
        199,
        129,
        125,
        242,
        233
      ],
      "accounts": [
        {
          "name": "observer",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  98,
                  115,
                  101,
                  114,
                  118,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "observer.wallet",
                "account": "observer"
              }
            ]
          }
        },
        {
          "name": "authority"
        },
        {
          "name": "registry",
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
        }
      ],
      "args": []
    },
    {
      "name": "suspendValidatorAvailability",
      "discriminator": [
        202,
        137,
        242,
        150,
        150,
        167,
        83,
        34
      ],
      "accounts": [
        {
          "name": "availability",
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
                  97,
                  118,
                  97,
                  105,
                  108,
                  97,
                  98,
                  105,
                  108,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "profile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "profile.wallet",
                "account": "validatorProfile"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
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
      "name": "sweepExpiredRights",
      "discriminator": [
        174,
        5,
        49,
        171,
        174,
        59,
        28,
        181
      ],
      "accounts": [
        {
          "name": "rights",
          "writable": true
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
                "account": "parcel"
              }
            ]
          }
        },
        {
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
              }
            ]
          }
        },
        {
          "name": "keeper",
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
      "name": "transferParcel",
      "docs": [
        "Transfer the ownership right of a parcel to `new_owner`. Only the",
        "current holder can sign (wallet holder or, via the identity overlay,",
        "the owner of an Identity PDA holder)."
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
          "name": "ownership",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
      "name": "unpauseProgram",
      "discriminator": [
        43,
        162,
        233,
        92,
        254,
        62,
        69,
        58
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "admin",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "updateDevice",
      "discriminator": [
        30,
        154,
        166,
        184,
        200,
        216,
        194,
        74
      ],
      "accounts": [
        {
          "name": "device",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  101,
                  118,
                  105,
                  99,
                  101,
                  95,
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
                "path": "device.owner",
                "account": "deviceIdentity"
              },
              {
                "kind": "account",
                "path": "device.deviceNonce",
                "account": "deviceIdentity"
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
          "name": "capabilities",
          "type": "u64"
        },
        {
          "name": "metadataRef",
          "type": "string"
        }
      ]
    },
    {
      "name": "updateInfrastructure",
      "docs": [
        "Set the parcel's infrastructure flag bitmask together with the canonical",
        "access digest produced by the off-chain validation engine. Holder-only.",
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
      "name": "updateJurisdiction",
      "discriminator": [
        172,
        83,
        224,
        77,
        240,
        153,
        228,
        24
      ],
      "accounts": [
        {
          "name": "jurisdiction",
          "writable": true
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "newVerificationKeyHash",
          "type": {
            "option": {
              "array": [
                "u8",
                32
              ]
            }
          }
        },
        {
          "name": "newRevocationRegistry",
          "type": {
            "option": "pubkey"
          }
        },
        {
          "name": "newStatus",
          "type": {
            "option": "u8"
          }
        }
      ]
    },
    {
      "name": "updateStatus",
      "docs": [
        "Update a parcel's status (e.g. for-sale). Holder-only."
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
          "name": "ownership",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "parcel"
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
    },
    {
      "name": "updateValidatorProfile",
      "discriminator": [
        31,
        20,
        102,
        26,
        128,
        98,
        167,
        153
      ],
      "accounts": [
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "wallet"
              }
            ]
          }
        },
        {
          "name": "wallet",
          "signer": true
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
          "name": "note",
          "type": "string"
        }
      ]
    },
    {
      "name": "updateVerificationKeyHash",
      "discriminator": [
        139,
        196,
        35,
        133,
        144,
        163,
        88,
        86
      ],
      "accounts": [
        {
          "name": "zoneSet",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  122,
                  111,
                  110,
                  101,
                  95,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet.zoneId",
                "account": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "ownershipRoot",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "authority",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "newVerificationKeyHash",
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
      "name": "verify3dModel",
      "docs": [
        "A registered validator attests a registered model's hash",
        "(claim → fact, same split as geometry verification)."
      ],
      "discriminator": [
        218,
        202,
        119,
        99,
        154,
        197,
        225,
        42
      ],
      "accounts": [
        {
          "name": "model",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  104,
                  114,
                  101,
                  101,
                  95,
                  100,
                  95,
                  109,
                  111,
                  100,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "model.asset",
                "account": "threeDModel"
              },
              {
                "kind": "account",
                "path": "model.modelVersion",
                "account": "threeDModel"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "verifyAndSlash",
      "discriminator": [
        247,
        117,
        243,
        191,
        71,
        156,
        117,
        175
      ],
      "accounts": [
        {
          "name": "slashingReport",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  108,
                  97,
                  115,
                  104,
                  105,
                  110,
                  103,
                  95,
                  114,
                  101,
                  112,
                  111,
                  114,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "slashingReport.reporter",
                "account": "slashingReport"
              },
              {
                "kind": "account",
                "path": "slashingReport.evidenceHash",
                "account": "slashingReport"
              }
            ]
          }
        },
        {
          "name": "offenderStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "offenderStake.validator",
                "account": "validatorStake"
              }
            ]
          }
        },
        {
          "name": "stakePool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "reporter",
          "writable": true
        },
        {
          "name": "treasury",
          "docs": [
            "Treasury PDA receiving slashed lamports. Constrained to a deterministic",
            "PDA derived from the authority registry — no one can redirect funds to",
            "an arbitrary wallet."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  116,
                  114,
                  101,
                  97,
                  115,
                  117,
                  114,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "verifyClaim",
      "discriminator": [
        35,
        121,
        58,
        82,
        51,
        132,
        99,
        113
      ],
      "accounts": [
        {
          "name": "claim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claim.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claim.claimId",
                "account": "claim"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "verifyCredential",
      "discriminator": [
        139,
        189,
        60,
        127,
        32,
        241,
        162,
        134
      ],
      "accounts": [
        {
          "name": "thresholdCredential",
          "writable": true
        },
        {
          "name": "nullifierRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  101,
                  100,
                  101,
                  110,
                  116,
                  105,
                  97,
                  108,
                  95,
                  110,
                  117,
                  108,
                  108,
                  105,
                  102,
                  105,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "thresholdCredential.nullifierHash",
                "account": "thresholdCredential"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "prover",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "instructions",
          "docs": [
            "(must be the Instructions sysvar holding this transaction's",
            "Ed25519 precompile attestation of the credential statement)."
          ]
        }
      ],
      "args": [
        {
          "name": "proofData",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "verifyCrossBorder",
      "discriminator": [
        84,
        43,
        145,
        123,
        243,
        248,
        57,
        11
      ],
      "accounts": [
        {
          "name": "crossBorderVerification",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  114,
                  111,
                  115,
                  115,
                  95,
                  98,
                  111,
                  114,
                  100,
                  101,
                  114,
                  95,
                  118,
                  101,
                  114,
                  105,
                  102,
                  105,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "crossBorderVerification.binding",
                "account": "crossBorderVerification"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "caller",
          "docs": [
            "Must be a registered validator or registry admin."
          ],
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "verifyDevice",
      "discriminator": [
        103,
        113,
        98,
        245,
        141,
        231,
        98,
        244
      ],
      "accounts": [
        {
          "name": "device",
          "writable": true
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "verifyGeometryVersion",
      "docs": [
        "A registered validator attests the version; `verified` flips when",
        "the append-time quorum threshold is reached (RFC-013 Phase B)."
      ],
      "discriminator": [
        71,
        36,
        152,
        213,
        205,
        229,
        137,
        125
      ],
      "accounts": [
        {
          "name": "geometryVersion",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  103,
                  101,
                  111,
                  109,
                  101,
                  116,
                  114,
                  121,
                  95,
                  118,
                  101,
                  114,
                  115,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "geometryVersion.asset",
                "account": "geometryVersion"
              },
              {
                "kind": "account",
                "path": "geometryVersion.version",
                "account": "geometryVersion"
              }
            ]
          }
        },
        {
          "name": "asset",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  112,
                  97,
                  116,
                  105,
                  97,
                  108,
                  95,
                  97,
                  115,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "asset.parcel",
                "account": "spatialAsset"
              }
            ]
          }
        },
        {
          "name": "profile",
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
                  112,
                  114,
                  111,
                  102,
                  105,
                  108,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "validator",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "verifyJurisdictionMembership",
      "discriminator": [
        130,
        50,
        171,
        217,
        217,
        92,
        207,
        78
      ],
      "accounts": [
        {
          "name": "binding",
          "writable": true
        },
        {
          "name": "jurisdiction"
        },
        {
          "name": "identity"
        },
        {
          "name": "validator",
          "signer": true
        },
        {
          "name": "registry",
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
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
      "name": "verifyOwnershipProof",
      "discriminator": [
        114,
        247,
        217,
        148,
        112,
        164,
        251,
        177
      ],
      "accounts": [
        {
          "name": "zoneSet",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  122,
                  111,
                  110,
                  101,
                  95,
                  115,
                  101,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet.zoneId",
                "account": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "ownershipRoot",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  119,
                  110,
                  101,
                  114,
                  115,
                  104,
                  105,
                  112,
                  95,
                  114,
                  111,
                  111,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "zoneSet"
              }
            ]
          }
        },
        {
          "name": "nullifierRecord",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  110,
                  117,
                  108,
                  108,
                  105,
                  102,
                  105,
                  101,
                  114
                ]
              },
              {
                "kind": "arg",
                "path": "nullifierHash"
              }
            ]
          }
        },
        {
          "name": "prover",
          "writable": true,
          "signer": true
        },
        {
          "name": "authority",
          "docs": [
            "Zone authority co-sign: every accepted proof is explicitly attested.",
            "`proof_data` is a framed v1 payload (RFC-011 §6.3): the prover's",
            "Ed25519 statement signature, the hash-pinned Groth16 verification",
            "key, and the Groth16 proof verified via the `alt_bn128` group",
            "operations syscalls. The production membership circuit and trusted",
            "setup remain gated on external audit."
          ],
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "instructions",
          "docs": [
            "(must be the Instructions sysvar holding this transaction's",
            "Ed25519 precompile attestation of the proof statement)."
          ]
        }
      ],
      "args": [
        {
          "name": "proofData",
          "type": "bytes"
        },
        {
          "name": "nullifierHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "rootVersion",
          "type": "u32"
        },
        {
          "name": "proofPurpose",
          "type": "string"
        },
        {
          "name": "disclosureType",
          "type": "u8"
        }
      ]
    },
    {
      "name": "voteChallenge",
      "discriminator": [
        198,
        76,
        44,
        76,
        134,
        76,
        143,
        98
      ],
      "accounts": [
        {
          "name": "challenge",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  97,
                  108,
                  108,
                  101,
                  110,
                  103,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "challenge.claim",
                "account": "challenge"
              },
              {
                "kind": "account",
                "path": "challenge.challenger",
                "account": "challenge"
              }
            ]
          }
        },
        {
          "name": "claimAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "claimAccount.parcel",
                "account": "claim"
              },
              {
                "kind": "account",
                "path": "claimAccount.claimId",
                "account": "claim"
              }
            ]
          }
        },
        {
          "name": "registry",
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
          "name": "validator",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "voteUphold",
          "type": "bool"
        }
      ]
    },
    {
      "name": "withdrawStake",
      "discriminator": [
        153,
        8,
        22,
        138,
        105,
        176,
        87,
        66
      ],
      "accounts": [
        {
          "name": "validatorStake",
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
                  115,
                  116,
                  97,
                  107,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "stakePool"
              },
              {
                "kind": "account",
                "path": "validator"
              }
            ]
          }
        },
        {
          "name": "stakePool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  116,
                  97,
                  107,
                  101,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "regionRegistry"
              }
            ]
          }
        },
        {
          "name": "regionRegistry",
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
          "name": "validator",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "amalgamationRecord",
      "discriminator": [
        4,
        242,
        238,
        37,
        228,
        163,
        250,
        90
      ]
    },
    {
      "name": "appeal",
      "discriminator": [
        155,
        196,
        80,
        143,
        64,
        220,
        198,
        177
      ]
    },
    {
      "name": "auditEntry",
      "discriminator": [
        254,
        88,
        234,
        107,
        205,
        16,
        148,
        113
      ]
    },
    {
      "name": "capabilityRestriction",
      "discriminator": [
        74,
        254,
        245,
        177,
        59,
        188,
        186,
        56
      ]
    },
    {
      "name": "challenge",
      "discriminator": [
        119,
        250,
        161,
        121,
        119,
        81,
        22,
        208
      ]
    },
    {
      "name": "claim",
      "discriminator": [
        155,
        70,
        22,
        176,
        123,
        215,
        246,
        102
      ]
    },
    {
      "name": "claimSessionTracker",
      "discriminator": [
        210,
        191,
        84,
        140,
        190,
        184,
        31,
        247
      ]
    },
    {
      "name": "coverageIncentive",
      "discriminator": [
        122,
        104,
        101,
        154,
        189,
        178,
        207,
        110
      ]
    },
    {
      "name": "credentialNullifier",
      "discriminator": [
        242,
        184,
        146,
        110,
        112,
        48,
        79,
        197
      ]
    },
    {
      "name": "credentialRequest",
      "discriminator": [
        13,
        77,
        28,
        51,
        251,
        145,
        231,
        29
      ]
    },
    {
      "name": "crossBorderBinding",
      "discriminator": [
        90,
        184,
        2,
        152,
        183,
        229,
        63,
        32
      ]
    },
    {
      "name": "crossBorderSpanRecord",
      "discriminator": [
        224,
        66,
        12,
        25,
        121,
        238,
        123,
        191
      ]
    },
    {
      "name": "crossBorderVerification",
      "discriminator": [
        183,
        6,
        175,
        22,
        50,
        174,
        166,
        63
      ]
    },
    {
      "name": "deviceIdentity",
      "discriminator": [
        6,
        98,
        9,
        166,
        229,
        128,
        194,
        58
      ]
    },
    {
      "name": "dispute",
      "discriminator": [
        36,
        49,
        241,
        67,
        40,
        36,
        241,
        74
      ]
    },
    {
      "name": "emergencyInjection",
      "discriminator": [
        101,
        247,
        85,
        71,
        70,
        156,
        253,
        173
      ]
    },
    {
      "name": "escrowRecord",
      "discriminator": [
        174,
        124,
        167,
        70,
        91,
        85,
        212,
        227
      ]
    },
    {
      "name": "evidence",
      "discriminator": [
        160,
        73,
        93,
        206,
        99,
        242,
        62,
        92
      ]
    },
    {
      "name": "evidenceArtifact",
      "discriminator": [
        18,
        40,
        202,
        16,
        128,
        245,
        57,
        135
      ]
    },
    {
      "name": "evidenceManifest",
      "discriminator": [
        9,
        190,
        128,
        86,
        64,
        17,
        31,
        5
      ]
    },
    {
      "name": "feePolicy",
      "discriminator": [
        98,
        183,
        86,
        160,
        232,
        222,
        82,
        45
      ]
    },
    {
      "name": "fraudReport",
      "discriminator": [
        136,
        69,
        45,
        181,
        210,
        24,
        121,
        177
      ]
    },
    {
      "name": "genesisRequest",
      "discriminator": [
        37,
        134,
        94,
        228,
        116,
        67,
        108,
        235
      ]
    },
    {
      "name": "geometryVersion",
      "discriminator": [
        78,
        143,
        35,
        194,
        97,
        213,
        195,
        216
      ]
    },
    {
      "name": "guardianClaim",
      "discriminator": [
        75,
        164,
        183,
        31,
        178,
        231,
        33,
        133
      ]
    },
    {
      "name": "identityRights",
      "discriminator": [
        80,
        133,
        246,
        210,
        78,
        97,
        163,
        46
      ]
    },
    {
      "name": "jurisdiction",
      "discriminator": [
        151,
        118,
        212,
        152,
        149,
        211,
        87,
        2
      ]
    },
    {
      "name": "jurisdictionBinding",
      "discriminator": [
        35,
        9,
        193,
        178,
        225,
        246,
        46,
        21
      ]
    },
    {
      "name": "nullifierRecord",
      "discriminator": [
        56,
        18,
        57,
        175,
        69,
        202,
        189,
        70
      ]
    },
    {
      "name": "observation",
      "discriminator": [
        109,
        190,
        190,
        95,
        28,
        172,
        243,
        74
      ]
    },
    {
      "name": "observationV2",
      "discriminator": [
        194,
        12,
        252,
        52,
        247,
        150,
        119,
        195
      ]
    },
    {
      "name": "observer",
      "discriminator": [
        82,
        255,
        234,
        217,
        166,
        201,
        80,
        72
      ]
    },
    {
      "name": "ownershipRoot",
      "discriminator": [
        130,
        74,
        117,
        24,
        60,
        232,
        70,
        225
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
      "name": "quorumConfig",
      "discriminator": [
        20,
        13,
        136,
        87,
        137,
        5,
        77,
        126
      ]
    },
    {
      "name": "quorumTally",
      "discriminator": [
        224,
        79,
        255,
        67,
        22,
        107,
        98,
        231
      ]
    },
    {
      "name": "quorumVote",
      "discriminator": [
        6,
        0,
        156,
        42,
        100,
        253,
        158,
        169
      ]
    },
    {
      "name": "resourceQuote",
      "discriminator": [
        12,
        173,
        242,
        254,
        102,
        42,
        31,
        5
      ]
    },
    {
      "name": "reviewCase",
      "discriminator": [
        91,
        9,
        100,
        255,
        116,
        51,
        53,
        93
      ]
    },
    {
      "name": "rewardAllocation",
      "discriminator": [
        126,
        230,
        173,
        145,
        43,
        165,
        30,
        112
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
      "name": "slashingReport",
      "discriminator": [
        73,
        176,
        94,
        36,
        3,
        159,
        87,
        218
      ]
    },
    {
      "name": "spatialAsset",
      "discriminator": [
        143,
        133,
        148,
        4,
        118,
        50,
        10,
        170
      ]
    },
    {
      "name": "spatialSnapshot",
      "discriminator": [
        156,
        60,
        119,
        78,
        61,
        63,
        161,
        245
      ]
    },
    {
      "name": "spatialSnapshotMeta",
      "discriminator": [
        123,
        54,
        165,
        171,
        172,
        35,
        63,
        23
      ]
    },
    {
      "name": "stakePool",
      "discriminator": [
        121,
        34,
        206,
        21,
        79,
        127,
        255,
        28
      ]
    },
    {
      "name": "subdivisionRecord",
      "discriminator": [
        9,
        214,
        108,
        50,
        166,
        44,
        134,
        32
      ]
    },
    {
      "name": "taskAssignment",
      "discriminator": [
        89,
        239,
        95,
        144,
        182,
        35,
        47,
        254
      ]
    },
    {
      "name": "taskEscrow",
      "discriminator": [
        209,
        72,
        197,
        54,
        17,
        55,
        3,
        187
      ]
    },
    {
      "name": "taskRequirement",
      "discriminator": [
        70,
        172,
        186,
        172,
        160,
        127,
        79,
        134
      ]
    },
    {
      "name": "threeDModel",
      "discriminator": [
        117,
        174,
        95,
        32,
        116,
        194,
        171,
        170
      ]
    },
    {
      "name": "threeDModelMeta",
      "discriminator": [
        231,
        80,
        5,
        178,
        80,
        198,
        86,
        7
      ]
    },
    {
      "name": "thresholdCredential",
      "discriminator": [
        217,
        121,
        146,
        123,
        84,
        119,
        65,
        79
      ]
    },
    {
      "name": "validatorActivityTracker",
      "discriminator": [
        45,
        51,
        146,
        237,
        228,
        13,
        159,
        133
      ]
    },
    {
      "name": "validatorAvailability",
      "discriminator": [
        113,
        127,
        178,
        236,
        25,
        220,
        94,
        89
      ]
    },
    {
      "name": "validatorCapability",
      "discriminator": [
        119,
        194,
        253,
        171,
        190,
        211,
        118,
        50
      ]
    },
    {
      "name": "validatorEndorsement",
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
    },
    {
      "name": "validatorNomination",
      "discriminator": [
        210,
        142,
        12,
        155,
        104,
        210,
        45,
        94
      ]
    },
    {
      "name": "validatorPresence",
      "discriminator": [
        49,
        114,
        241,
        63,
        232,
        95,
        50,
        90
      ]
    },
    {
      "name": "validatorProfile",
      "discriminator": [
        50,
        223,
        146,
        114,
        220,
        208,
        127,
        86
      ]
    },
    {
      "name": "validatorRegistry",
      "discriminator": [
        168,
        113,
        195,
        186,
        62,
        121,
        163,
        230
      ]
    },
    {
      "name": "validatorRelationshipEdge",
      "discriminator": [
        128,
        240,
        223,
        172,
        123,
        148,
        81,
        48
      ]
    },
    {
      "name": "validatorReputation",
      "discriminator": [
        151,
        57,
        248,
        51,
        14,
        124,
        177,
        47
      ]
    },
    {
      "name": "validatorStake",
      "discriminator": [
        95,
        210,
        178,
        207,
        78,
        5,
        247,
        29
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
    },
    {
      "name": "verificationAttestation",
      "discriminator": [
        231,
        126,
        92,
        51,
        84,
        178,
        81,
        242
      ]
    },
    {
      "name": "verificationSession",
      "discriminator": [
        247,
        69,
        199,
        26,
        112,
        195,
        243,
        254
      ]
    },
    {
      "name": "verificationTask",
      "discriminator": [
        96,
        182,
        137,
        216,
        62,
        108,
        74,
        190
      ]
    },
    {
      "name": "worldRegistry",
      "discriminator": [
        222,
        41,
        153,
        93,
        64,
        60,
        109,
        27
      ]
    },
    {
      "name": "zoneSet",
      "discriminator": [
        57,
        110,
        98,
        208,
        4,
        53,
        98,
        77
      ]
    }
  ],
  "events": [
    {
      "name": "appealReviewOpened",
      "discriminator": [
        158,
        228,
        47,
        141,
        115,
        86,
        151,
        4
      ]
    },
    {
      "name": "auditEntryRecorded",
      "discriminator": [
        216,
        172,
        160,
        149,
        136,
        44,
        191,
        201
      ]
    },
    {
      "name": "capabilityRestrictionApplied",
      "discriminator": [
        224,
        226,
        118,
        114,
        180,
        19,
        233,
        124
      ]
    },
    {
      "name": "capabilityRestrictionLifted",
      "discriminator": [
        227,
        47,
        169,
        65,
        35,
        29,
        146,
        240
      ]
    },
    {
      "name": "challengeFiled",
      "discriminator": [
        76,
        64,
        122,
        85,
        174,
        3,
        91,
        71
      ]
    },
    {
      "name": "challengeResolved",
      "discriminator": [
        100,
        153,
        38,
        123,
        172,
        250,
        166,
        105
      ]
    },
    {
      "name": "challengeVoteRecorded",
      "discriminator": [
        154,
        43,
        244,
        154,
        63,
        108,
        57,
        201
      ]
    },
    {
      "name": "claimCreated",
      "discriminator": [
        12,
        137,
        189,
        74,
        127,
        86,
        118,
        130
      ]
    },
    {
      "name": "claimRejected",
      "discriminator": [
        108,
        107,
        44,
        113,
        45,
        127,
        6,
        77
      ]
    },
    {
      "name": "claimVerified",
      "discriminator": [
        90,
        196,
        170,
        218,
        88,
        102,
        26,
        4
      ]
    },
    {
      "name": "countryAllocated",
      "discriminator": [
        194,
        247,
        216,
        194,
        79,
        102,
        241,
        133
      ]
    },
    {
      "name": "coverageIncentiveSet",
      "discriminator": [
        49,
        181,
        252,
        116,
        211,
        58,
        238,
        115
      ]
    },
    {
      "name": "credentialIssued",
      "discriminator": [
        194,
        216,
        28,
        159,
        89,
        29,
        72,
        177
      ]
    },
    {
      "name": "credentialRequested",
      "discriminator": [
        240,
        139,
        197,
        54,
        209,
        3,
        199,
        122
      ]
    },
    {
      "name": "credentialSigned",
      "discriminator": [
        14,
        163,
        97,
        76,
        98,
        144,
        224,
        42
      ]
    },
    {
      "name": "credentialVerified",
      "discriminator": [
        242,
        32,
        25,
        117,
        249,
        65,
        220,
        214
      ]
    },
    {
      "name": "crossBorderBindingCreated",
      "discriminator": [
        16,
        33,
        97,
        130,
        88,
        12,
        166,
        16
      ]
    },
    {
      "name": "crossBorderBindingStatusChanged",
      "discriminator": [
        107,
        106,
        174,
        210,
        243,
        208,
        12,
        80
      ]
    },
    {
      "name": "crossBorderIdentityBound",
      "discriminator": [
        19,
        25,
        251,
        19,
        198,
        99,
        75,
        217
      ]
    },
    {
      "name": "crossBorderIdentityRebound",
      "discriminator": [
        7,
        57,
        174,
        104,
        169,
        253,
        101,
        121
      ]
    },
    {
      "name": "crossBorderLinked",
      "discriminator": [
        85,
        22,
        30,
        5,
        176,
        226,
        48,
        33
      ]
    },
    {
      "name": "crossBorderRevoked",
      "discriminator": [
        56,
        162,
        138,
        171,
        244,
        65,
        209,
        11
      ]
    },
    {
      "name": "crossBorderVerificationRecorded",
      "discriminator": [
        255,
        211,
        5,
        15,
        188,
        194,
        90,
        103
      ]
    },
    {
      "name": "crossBorderVerified",
      "discriminator": [
        22,
        74,
        185,
        189,
        26,
        183,
        146,
        214
      ]
    },
    {
      "name": "deviceCalibrationSet",
      "discriminator": [
        92,
        152,
        193,
        158,
        131,
        143,
        73,
        231
      ]
    },
    {
      "name": "deviceKeyRotated",
      "discriminator": [
        220,
        43,
        143,
        107,
        183,
        120,
        161,
        59
      ]
    },
    {
      "name": "deviceRegistered",
      "discriminator": [
        221,
        90,
        2,
        153,
        72,
        98,
        71,
        181
      ]
    },
    {
      "name": "deviceStatusChanged",
      "discriminator": [
        47,
        92,
        8,
        190,
        132,
        76,
        125,
        174
      ]
    },
    {
      "name": "deviceUpdated",
      "discriminator": [
        19,
        61,
        200,
        102,
        163,
        178,
        2,
        158
      ]
    },
    {
      "name": "deviceVerified",
      "discriminator": [
        119,
        100,
        199,
        37,
        44,
        117,
        220,
        189
      ]
    },
    {
      "name": "disputeAdjudicated",
      "discriminator": [
        181,
        107,
        82,
        172,
        183,
        3,
        201,
        187
      ]
    },
    {
      "name": "disputeCancelled",
      "discriminator": [
        38,
        80,
        189,
        225,
        112,
        190,
        156,
        178
      ]
    },
    {
      "name": "disputeFiled",
      "discriminator": [
        201,
        119,
        206,
        64,
        219,
        44,
        249,
        153
      ]
    },
    {
      "name": "emergencyInjectionExecuted",
      "discriminator": [
        66,
        71,
        186,
        169,
        177,
        166,
        226,
        229
      ]
    },
    {
      "name": "emergencyInjectionQueued",
      "discriminator": [
        85,
        13,
        68,
        71,
        13,
        119,
        229,
        206
      ]
    },
    {
      "name": "equivocationReported",
      "discriminator": [
        136,
        91,
        235,
        5,
        219,
        141,
        157,
        248
      ]
    },
    {
      "name": "escrowAccepted",
      "discriminator": [
        129,
        122,
        76,
        235,
        127,
        11,
        32,
        165
      ]
    },
    {
      "name": "escrowCancelled",
      "discriminator": [
        98,
        241,
        195,
        122,
        213,
        0,
        162,
        161
      ]
    },
    {
      "name": "escrowCreated",
      "discriminator": [
        70,
        127,
        105,
        102,
        92,
        97,
        7,
        173
      ]
    },
    {
      "name": "escrowDeposited",
      "discriminator": [
        28,
        193,
        105,
        27,
        40,
        101,
        65,
        211
      ]
    },
    {
      "name": "escrowDisputed",
      "discriminator": [
        132,
        73,
        81,
        200,
        177,
        51,
        128,
        18
      ]
    },
    {
      "name": "escrowExpired",
      "discriminator": [
        189,
        22,
        170,
        250,
        75,
        218,
        58,
        112
      ]
    },
    {
      "name": "escrowSettled",
      "discriminator": [
        97,
        27,
        150,
        55,
        203,
        179,
        173,
        23
      ]
    },
    {
      "name": "evidenceAdded",
      "discriminator": [
        124,
        213,
        122,
        98,
        248,
        253,
        235,
        89
      ]
    },
    {
      "name": "evidenceArtifactAdded",
      "discriminator": [
        17,
        174,
        204,
        232,
        109,
        240,
        149,
        44
      ]
    },
    {
      "name": "evidenceManifestSubmitted",
      "discriminator": [
        239,
        142,
        224,
        208,
        67,
        1,
        245,
        201
      ]
    },
    {
      "name": "feePolicySet",
      "discriminator": [
        201,
        223,
        52,
        122,
        8,
        112,
        188,
        98
      ]
    },
    {
      "name": "fraudAppealDecided",
      "discriminator": [
        24,
        216,
        217,
        225,
        99,
        196,
        222,
        160
      ]
    },
    {
      "name": "fraudAppealFiled",
      "discriminator": [
        251,
        135,
        229,
        53,
        160,
        103,
        69,
        244
      ]
    },
    {
      "name": "fraudReportSubmitted",
      "discriminator": [
        18,
        15,
        109,
        93,
        27,
        149,
        200,
        83
      ]
    },
    {
      "name": "fraudReviewFinalized",
      "discriminator": [
        196,
        214,
        48,
        219,
        18,
        103,
        207,
        227
      ]
    },
    {
      "name": "fraudReviewOpened",
      "discriminator": [
        35,
        186,
        232,
        174,
        11,
        57,
        157,
        179
      ]
    },
    {
      "name": "fraudVoteCast",
      "discriminator": [
        30,
        73,
        152,
        83,
        81,
        88,
        81,
        137
      ]
    },
    {
      "name": "genesisConfirmed",
      "discriminator": [
        216,
        113,
        83,
        241,
        163,
        230,
        0,
        236
      ]
    },
    {
      "name": "genesisFinalized",
      "discriminator": [
        95,
        88,
        165,
        229,
        106,
        30,
        73,
        160
      ]
    },
    {
      "name": "genesisRequested",
      "discriminator": [
        117,
        141,
        248,
        2,
        9,
        205,
        148,
        122
      ]
    },
    {
      "name": "geometryVersionAppended",
      "discriminator": [
        80,
        64,
        84,
        239,
        254,
        110,
        6,
        57
      ]
    },
    {
      "name": "geometryVersionAttested",
      "discriminator": [
        200,
        92,
        5,
        221,
        66,
        252,
        209,
        72
      ]
    },
    {
      "name": "geometryVersionVerified",
      "discriminator": [
        169,
        60,
        89,
        147,
        17,
        205,
        122,
        243
      ]
    },
    {
      "name": "guardianClaimCreated",
      "discriminator": [
        101,
        249,
        220,
        104,
        98,
        79,
        23,
        165
      ]
    },
    {
      "name": "guardianClaimDisputed",
      "discriminator": [
        54,
        233,
        20,
        187,
        63,
        136,
        17,
        35
      ]
    },
    {
      "name": "guardianClaimResolved",
      "discriminator": [
        80,
        117,
        102,
        68,
        169,
        247,
        166,
        106
      ]
    },
    {
      "name": "identityRightGranted",
      "discriminator": [
        35,
        127,
        165,
        2,
        72,
        121,
        16,
        185
      ]
    },
    {
      "name": "identityRightRevoked",
      "discriminator": [
        73,
        23,
        31,
        237,
        100,
        111,
        70,
        123
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
      "name": "judgmentExecuted",
      "discriminator": [
        142,
        230,
        47,
        185,
        130,
        105,
        52,
        48
      ]
    },
    {
      "name": "jurisdictionIdentityRevoked",
      "discriminator": [
        38,
        53,
        194,
        162,
        49,
        0,
        25,
        163
      ]
    },
    {
      "name": "jurisdictionMembershipVerified",
      "discriminator": [
        18,
        92,
        32,
        71,
        68,
        180,
        181,
        9
      ]
    },
    {
      "name": "jurisdictionRegistered",
      "discriminator": [
        128,
        39,
        215,
        134,
        189,
        176,
        106,
        245
      ]
    },
    {
      "name": "jurisdictionUpdated",
      "discriminator": [
        231,
        40,
        254,
        68,
        169,
        51,
        157,
        55
      ]
    },
    {
      "name": "nominationConfirmed",
      "discriminator": [
        231,
        5,
        164,
        206,
        158,
        29,
        183,
        191
      ]
    },
    {
      "name": "observationSubmitted",
      "discriminator": [
        108,
        255,
        169,
        65,
        60,
        235,
        203,
        188
      ]
    },
    {
      "name": "observationV2Submitted",
      "discriminator": [
        220,
        139,
        72,
        84,
        9,
        67,
        34,
        81
      ]
    },
    {
      "name": "observerRegistered",
      "discriminator": [
        33,
        248,
        190,
        137,
        191,
        38,
        49,
        56
      ]
    },
    {
      "name": "observerStatusChanged",
      "discriminator": [
        144,
        198,
        36,
        100,
        48,
        176,
        191,
        203
      ]
    },
    {
      "name": "ownershipProofVerified",
      "discriminator": [
        133,
        80,
        213,
        132,
        28,
        110,
        139,
        191
      ]
    },
    {
      "name": "ownershipRootUpdated",
      "discriminator": [
        194,
        16,
        79,
        126,
        75,
        137,
        150,
        54
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
      "name": "parcelFrozen",
      "discriminator": [
        1,
        224,
        62,
        2,
        118,
        17,
        202,
        217
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
      "name": "parcelSubdivided",
      "discriminator": [
        226,
        169,
        51,
        12,
        208,
        85,
        159,
        177
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
      "name": "parcelsAmalgamated",
      "discriminator": [
        223,
        144,
        63,
        29,
        48,
        176,
        253,
        74
      ]
    },
    {
      "name": "programPaused",
      "discriminator": [
        247,
        9,
        236,
        38,
        105,
        35,
        112,
        115
      ]
    },
    {
      "name": "programUnpaused",
      "discriminator": [
        193,
        183,
        146,
        52,
        180,
        17,
        21,
        26
      ]
    },
    {
      "name": "proofVersionInvalidated",
      "discriminator": [
        218,
        103,
        154,
        148,
        120,
        25,
        121,
        128
      ]
    },
    {
      "name": "quorumConfigSet",
      "discriminator": [
        127,
        51,
        134,
        210,
        179,
        235,
        221,
        116
      ]
    },
    {
      "name": "quorumReachabilityChecked",
      "discriminator": [
        50,
        201,
        111,
        245,
        128,
        167,
        112,
        240
      ]
    },
    {
      "name": "quorumReached",
      "discriminator": [
        234,
        207,
        54,
        50,
        183,
        135,
        255,
        213
      ]
    },
    {
      "name": "quorumVoteCast",
      "discriminator": [
        102,
        60,
        79,
        169,
        52,
        132,
        255,
        227
      ]
    },
    {
      "name": "registryCreated",
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
      "name": "reportDismissed",
      "discriminator": [
        181,
        63,
        248,
        138,
        116,
        100,
        43,
        164
      ]
    },
    {
      "name": "reputationSlashed",
      "discriminator": [
        171,
        140,
        27,
        238,
        91,
        224,
        11,
        6
      ]
    },
    {
      "name": "rewardsClaimed",
      "discriminator": [
        75,
        98,
        88,
        18,
        219,
        112,
        88,
        121
      ]
    },
    {
      "name": "rewardsDistributed",
      "discriminator": [
        11,
        43,
        154,
        0,
        229,
        9,
        116,
        85
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
      "name": "rightRenewed",
      "discriminator": [
        55,
        0,
        155,
        253,
        159,
        84,
        186,
        197
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
      "name": "rightStatusTransition",
      "discriminator": [
        75,
        178,
        7,
        36,
        17,
        198,
        215,
        153
      ]
    },
    {
      "name": "rightsMigrated",
      "discriminator": [
        105,
        144,
        146,
        255,
        89,
        164,
        49,
        238
      ]
    },
    {
      "name": "rightsMigrationComplete",
      "discriminator": [
        244,
        52,
        186,
        231,
        72,
        181,
        101,
        90
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
      "name": "slashingAppealed",
      "discriminator": [
        40,
        35,
        123,
        102,
        1,
        153,
        139,
        77
      ]
    },
    {
      "name": "spatialAssetCreated",
      "discriminator": [
        92,
        105,
        247,
        37,
        169,
        73,
        194,
        173
      ]
    },
    {
      "name": "spatialSnapshotCreated",
      "discriminator": [
        74,
        109,
        89,
        169,
        182,
        165,
        45,
        64
      ]
    },
    {
      "name": "stakeDeposited",
      "discriminator": [
        69,
        152,
        144,
        109,
        232,
        34,
        225,
        19
      ]
    },
    {
      "name": "stakePoolCreated",
      "discriminator": [
        80,
        44,
        243,
        249,
        137,
        72,
        229,
        122
      ]
    },
    {
      "name": "stakeWithdrawn",
      "discriminator": [
        33,
        120,
        159,
        58,
        140,
        255,
        174,
        79
      ]
    },
    {
      "name": "successionParcelsTransferred",
      "discriminator": [
        185,
        213,
        67,
        115,
        214,
        219,
        212,
        249
      ]
    },
    {
      "name": "taskAssigned",
      "discriminator": [
        67,
        8,
        87,
        59,
        111,
        91,
        204,
        170
      ]
    },
    {
      "name": "taskCancelled",
      "discriminator": [
        158,
        101,
        220,
        187,
        16,
        141,
        141,
        64
      ]
    },
    {
      "name": "taskEscrowFunded",
      "discriminator": [
        212,
        62,
        115,
        234,
        122,
        182,
        18,
        251
      ]
    },
    {
      "name": "taskEscrowRefunded",
      "discriminator": [
        148,
        42,
        136,
        67,
        153,
        136,
        73,
        19
      ]
    },
    {
      "name": "taskRequirementAdded",
      "discriminator": [
        210,
        120,
        31,
        247,
        164,
        100,
        118,
        172
      ]
    },
    {
      "name": "taskResourcesQuoted",
      "discriminator": [
        80,
        109,
        170,
        72,
        46,
        159,
        229,
        116
      ]
    },
    {
      "name": "taskResultSubmitted",
      "discriminator": [
        138,
        65,
        217,
        42,
        15,
        246,
        83,
        32
      ]
    },
    {
      "name": "taskRewardClaimed",
      "discriminator": [
        185,
        139,
        195,
        109,
        20,
        19,
        161,
        24
      ]
    },
    {
      "name": "taskRouted",
      "discriminator": [
        180,
        115,
        100,
        224,
        93,
        113,
        123,
        103
      ]
    },
    {
      "name": "threeDModelRegistered",
      "discriminator": [
        107,
        244,
        21,
        221,
        200,
        103,
        215,
        89
      ]
    },
    {
      "name": "threeDModelVerified",
      "discriminator": [
        13,
        186,
        124,
        19,
        47,
        165,
        98,
        63
      ]
    },
    {
      "name": "unbondingInitiated",
      "discriminator": [
        119,
        120,
        72,
        229,
        146,
        100,
        18,
        106
      ]
    },
    {
      "name": "validatorActiveSet",
      "discriminator": [
        140,
        223,
        60,
        104,
        129,
        217,
        250,
        227
      ]
    },
    {
      "name": "validatorAdded",
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
      "name": "validatorAvailabilityChanged",
      "discriminator": [
        149,
        204,
        226,
        1,
        214,
        220,
        195,
        72
      ]
    },
    {
      "name": "validatorCapabilityDeclared",
      "discriminator": [
        147,
        147,
        83,
        152,
        88,
        217,
        69,
        182
      ]
    },
    {
      "name": "validatorEndorsed",
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
      "name": "validatorJailed",
      "discriminator": [
        219,
        171,
        53,
        20,
        142,
        121,
        114,
        72
      ]
    },
    {
      "name": "validatorJurisdictionSet",
      "discriminator": [
        146,
        222,
        204,
        105,
        21,
        42,
        7,
        147
      ]
    },
    {
      "name": "validatorNominated",
      "discriminator": [
        112,
        168,
        111,
        7,
        169,
        135,
        174,
        211
      ]
    },
    {
      "name": "validatorNominationFinalized",
      "discriminator": [
        102,
        105,
        161,
        233,
        236,
        57,
        36,
        155
      ]
    },
    {
      "name": "validatorPresenceUpdated",
      "discriminator": [
        34,
        86,
        135,
        229,
        6,
        7,
        121,
        145
      ]
    },
    {
      "name": "validatorProfileInitialized",
      "discriminator": [
        249,
        2,
        119,
        241,
        158,
        220,
        204,
        172
      ]
    },
    {
      "name": "validatorProfileTierChanged",
      "discriminator": [
        208,
        17,
        40,
        237,
        187,
        190,
        27,
        108
      ]
    },
    {
      "name": "validatorProfileUpdated",
      "discriminator": [
        40,
        88,
        228,
        225,
        76,
        214,
        134,
        24
      ]
    },
    {
      "name": "validatorRelationshipEdgeCreated",
      "discriminator": [
        32,
        95,
        29,
        62,
        146,
        177,
        5,
        100
      ]
    },
    {
      "name": "validatorRemoved",
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
    },
    {
      "name": "validatorReputationInitialized",
      "discriminator": [
        214,
        137,
        171,
        142,
        137,
        238,
        192,
        175
      ]
    },
    {
      "name": "validatorSlashed",
      "discriminator": [
        1,
        160,
        99,
        18,
        25,
        42,
        5,
        213
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
    },
    {
      "name": "verificationAttestationSubmitted",
      "discriminator": [
        67,
        110,
        129,
        218,
        46,
        207,
        63,
        162
      ]
    },
    {
      "name": "verificationKeyUpdated",
      "discriminator": [
        186,
        219,
        161,
        35,
        204,
        1,
        3,
        207
      ]
    },
    {
      "name": "verificationSessionClosed",
      "discriminator": [
        63,
        201,
        50,
        148,
        176,
        169,
        203,
        132
      ]
    },
    {
      "name": "verificationSessionOpened",
      "discriminator": [
        97,
        6,
        127,
        245,
        102,
        6,
        135,
        151
      ]
    },
    {
      "name": "verificationTaskCreated",
      "discriminator": [
        225,
        37,
        127,
        148,
        192,
        238,
        23,
        87
      ]
    },
    {
      "name": "worldRegistryCreated",
      "discriminator": [
        18,
        158,
        240,
        20,
        48,
        170,
        173,
        200
      ]
    },
    {
      "name": "zoneSetRegistered",
      "discriminator": [
        83,
        7,
        29,
        172,
        101,
        159,
        212,
        6
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
      "msg": "Identity owner does not match the acting holder"
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
      "name": "validatorOwnsAsset",
      "msg": "A validator cannot be the owner of the asset being validated"
    },
    {
      "code": 6035,
      "name": "vaultAlreadyExists",
      "msg": "Vault already exists for this subject"
    },
    {
      "code": 6036,
      "name": "vaultNotFound",
      "msg": "Vault not found"
    },
    {
      "code": 6037,
      "name": "thresholdExceedsHolders",
      "msg": "Threshold exceeds the number of shard holders"
    },
    {
      "code": 6038,
      "name": "notShardHolder",
      "msg": "Signer is not a shard holder for this vault"
    },
    {
      "code": 6039,
      "name": "notActiveValidator",
      "msg": "Signer is not an active validator in this vault"
    },
    {
      "code": 6040,
      "name": "ciphertextHashRequired",
      "msg": "Ciphertext hash cannot be all zeros"
    },
    {
      "code": 6041,
      "name": "cidRequired",
      "msg": "Ciphertext CID cannot be empty"
    },
    {
      "code": 6042,
      "name": "expiryTooFar",
      "msg": "Expiry must be within 24 hours from now"
    },
    {
      "code": 6043,
      "name": "expiryInPast",
      "msg": "Expiry must be in the future"
    },
    {
      "code": 6044,
      "name": "nonceRequired",
      "msg": "Nonce cannot be all zeros"
    },
    {
      "code": 6045,
      "name": "rotationNotFound",
      "msg": "No pending rotation exists for this vault"
    },
    {
      "code": 6046,
      "name": "rotationAlreadyFinalized",
      "msg": "Rotation has already been executed or cancelled"
    },
    {
      "code": 6047,
      "name": "rotationNotYetEffective",
      "msg": "Rotation time lock has not yet expired"
    },
    {
      "code": 6048,
      "name": "quorumNotMetForRotation",
      "msg": "Not enough endorsements for rotation (need ceil(2n/3))"
    },
    {
      "code": 6049,
      "name": "alreadyEndorsedRotation",
      "msg": "Validator has already endorsed this rotation"
    },
    {
      "code": 6050,
      "name": "selfEndorsementNotAllowed",
      "msg": "Initiator cannot endorse their own rotation"
    },
    {
      "code": 6051,
      "name": "pendingRotationExists",
      "msg": "A pending rotation already exists for this vault"
    },
    {
      "code": 6052,
      "name": "pingIntervalNotElapsed",
      "msg": "Ping interval has not yet elapsed"
    },
    {
      "code": 6053,
      "name": "algorithmNotSupported",
      "msg": "Encryption algorithm is not supported"
    },
    {
      "code": 6054,
      "name": "tooManyStorageUris",
      "msg": "Storage URIs exceed the maximum count"
    },
    {
      "code": 6055,
      "name": "tooManyShardHolders",
      "msg": "Shard holders exceed the maximum count"
    },
    {
      "code": 6056,
      "name": "nonceAlreadyUsed",
      "msg": "This nonce has already been used for this vault"
    },
    {
      "code": 6057,
      "name": "notAuthorizedToCreate",
      "msg": "Only the registry admin or subject's recovery wallet can create a vault"
    },
    {
      "code": 6058,
      "name": "notAuthorizedToCancel",
      "msg": "Only the admin or initiator can cancel a rotation"
    },
    {
      "code": 6059,
      "name": "newThresholdExceedsHolders",
      "msg": "New threshold exceeds the number of new shard holders"
    },
    {
      "code": 6060,
      "name": "invalidDisputeStatus",
      "msg": "Invalid dispute status for this operation"
    },
    {
      "code": 6061,
      "name": "disputeExpired",
      "msg": "Dispute has expired (90-day window elapsed)"
    },
    {
      "code": 6062,
      "name": "invalidDisputeOutcome",
      "msg": "Invalid dispute outcome"
    },
    {
      "code": 6063,
      "name": "invalidEscrowStatus",
      "msg": "Invalid escrow status for this operation"
    },
    {
      "code": 6064,
      "name": "invalidEscrowAmount",
      "msg": "Escrow amount is outside the allowed range"
    },
    {
      "code": 6065,
      "name": "selfDealingNotAllowed",
      "msg": "Buyer wallet cannot be the same as the seller"
    },
    {
      "code": 6066,
      "name": "notDesignatedBuyer",
      "msg": "Signer is not the designated buyer for this escrow"
    },
    {
      "code": 6067,
      "name": "notDesignatedSeller",
      "msg": "Signer is not the designated seller for this escrow"
    },
    {
      "code": 6068,
      "name": "depositExceedsAmount",
      "msg": "Deposit would exceed the escrow amount"
    },
    {
      "code": 6069,
      "name": "insufficientDeposit",
      "msg": "Full deposit not yet received — seller cannot accept"
    },
    {
      "code": 6070,
      "name": "settlementNotYetEffective",
      "msg": "Settlement window has not yet expired"
    },
    {
      "code": 6071,
      "name": "parcelStillOwnedBySeller",
      "msg": "Seller no longer owns the parcel at settlement time"
    },
    {
      "code": 6072,
      "name": "cancelWindowExpired",
      "msg": "Buyer grace period has expired — cannot cancel"
    },
    {
      "code": 6073,
      "name": "cancelWindowNotExpired",
      "msg": "Cancel window has not yet expired — escrow not stale"
    },
    {
      "code": 6074,
      "name": "depositExists",
      "msg": "Cannot cancel: deposit already exists"
    },
    {
      "code": 6075,
      "name": "notPartyToEscrow",
      "msg": "Signer is not a party to this escrow"
    },
    {
      "code": 6076,
      "name": "parcelAlreadyInEscrow",
      "msg": "Parcel is already in an escrow"
    },
    {
      "code": 6077,
      "name": "escrowAmountTooLow",
      "msg": "Minimum escrow amount is 0.1 SOL"
    },
    {
      "code": 6078,
      "name": "escrowAmountTooHigh",
      "msg": "Maximum escrow amount is 1,000,000 SOL"
    },
    {
      "code": 6079,
      "name": "emptyBuyer",
      "msg": "Buyer wallet address is required"
    },
    {
      "code": 6080,
      "name": "cannotRenewRevokedRight",
      "msg": "Cannot renew a revoked right"
    },
    {
      "code": 6081,
      "name": "cannotRenewAlreadyRenewed",
      "msg": "Cannot renew an already-renewed right"
    },
    {
      "code": 6082,
      "name": "renewalMustExtendExpiry",
      "msg": "Renewal must extend the expiry, not shorten it"
    },
    {
      "code": 6083,
      "name": "invalidRightStatus",
      "msg": "Invalid right status for this operation"
    },
    {
      "code": 6084,
      "name": "permanentRightNotSweepable",
      "msg": "Permanent rights (expires_at=0) cannot be swept"
    },
    {
      "code": 6085,
      "name": "conditionDeadlineAfterExpiry",
      "msg": "Condition deadline must be before the expiry"
    },
    {
      "code": 6086,
      "name": "invalidGracePeriod",
      "msg": "Invalid grace period"
    },
    {
      "code": 6087,
      "name": "invalidProofData",
      "msg": "Invalid proof data"
    },
    {
      "code": 6088,
      "name": "invalidJurisdictionStatus",
      "msg": "Jurisdiction is not active"
    },
    {
      "code": 6089,
      "name": "bindingRevoked",
      "msg": "Binding has been revoked"
    },
    {
      "code": 6090,
      "name": "bindingExpired",
      "msg": "Binding has expired"
    },
    {
      "code": 6091,
      "name": "bindingAlreadyRevoked",
      "msg": "Binding is already revoked"
    },
    {
      "code": 6092,
      "name": "nullifierCollision",
      "msg": "Nullifier collision detected"
    },
    {
      "code": 6093,
      "name": "invalidParcelStatus",
      "msg": "Parcel is not in the correct status for this operation"
    },
    {
      "code": 6094,
      "name": "invalidGeometryHash",
      "msg": "Parcel geometry hash is invalid"
    },
    {
      "code": 6095,
      "name": "rightsMigrationFailed",
      "msg": "Rights migration failed"
    },
    {
      "code": 6096,
      "name": "attestationMigrationFailed",
      "msg": "Attestation migration failed"
    },
    {
      "code": 6097,
      "name": "subdivisionRecordExists",
      "msg": "Subdivision record already exists for this pair"
    },
    {
      "code": 6098,
      "name": "amalgamationRecordExists",
      "msg": "Amalgamation record already exists for this pair"
    },
    {
      "code": 6099,
      "name": "insufficientStake",
      "msg": "Insufficient stake for this operation"
    },
    {
      "code": 6100,
      "name": "stakeAlreadyActive",
      "msg": "Stake is already active — cannot deposit again"
    },
    {
      "code": 6101,
      "name": "unbondingInProgress",
      "msg": "Unbonding is in progress — cannot deposit or withdraw"
    },
    {
      "code": 6102,
      "name": "unbondingNotComplete",
      "msg": "Unbonding period has not yet completed"
    },
    {
      "code": 6103,
      "name": "selfReportNotAllowed",
      "msg": "Self-reporting is not allowed"
    },
    {
      "code": 6104,
      "name": "guardianshipGraceTooShort",
      "msg": "Guardianship grace period is below the 90-day minimum"
    },
    {
      "code": 6105,
      "name": "guardianshipThresholdTooLow",
      "msg": "Guardianship requires at least 3 validator endorsements"
    },
    {
      "code": 6106,
      "name": "nullifierAlreadyUsed",
      "msg": "Proof has already been used (nullifier recorded)"
    },
    {
      "code": 6107,
      "name": "rootVersionMismatch",
      "msg": "Proof references a stale or unknown root version"
    },
    {
      "code": 6108,
      "name": "unauthorizedZoneAuthority",
      "msg": "Signer is not the zone authority"
    },
    {
      "code": 6109,
      "name": "emptyZoneSet",
      "msg": "Zone has no commitments to prove against"
    },
    {
      "code": 6110,
      "name": "invalidProofPurpose",
      "msg": "Proof purpose is missing or too long"
    },
    {
      "code": 6111,
      "name": "proofTooLarge",
      "msg": "Proof data is empty or exceeds the maximum size"
    },
    {
      "code": 6112,
      "name": "invalidDisclosureType",
      "msg": "Disclosure type must be 0 (membership), 1 (range), or 2 (count)"
    },
    {
      "code": 6113,
      "name": "invalidOffenseType",
      "msg": "Offense type must be 0 (equivocation), 1 (liveness), or 2 (collusion)"
    },
    {
      "code": 6114,
      "name": "invalidRegistryMode",
      "msg": "Registry is not in the required mode for this operation"
    },
    {
      "code": 6115,
      "name": "noProposalFound",
      "msg": "No validator endorsement proposal exists for this candidate"
    },
    {
      "code": 6116,
      "name": "wrongOnboardingStage",
      "msg": "Wrong onboarding stage for this bootstrap operation"
    },
    {
      "code": 6117,
      "name": "sponsorCannotConfirm",
      "msg": "Sponsor cannot confirm their own nomination"
    },
    {
      "code": 6118,
      "name": "invalidCountryCode",
      "msg": "Invalid ISO 3166-1 alpha-2 country code"
    },
    {
      "code": 6119,
      "name": "countryAlreadyAllocated",
      "msg": "Country code has already been allocated"
    },
    {
      "code": 6120,
      "name": "countryNotAllocated",
      "msg": "Country is not allocated to this admin"
    },
    {
      "code": 6121,
      "name": "confirmersNotDiverseEnough",
      "msg": "Genesis confirmers are not from enough distinct countries"
    },
    {
      "code": 6122,
      "name": "emergencyTimelockNotElapsed",
      "msg": "Emergency injection timelock has not yet elapsed"
    },
    {
      "code": 6123,
      "name": "insufficientEndorsements",
      "msg": "Insufficient validator endorsements for threshold credential"
    },
    {
      "code": 6124,
      "name": "programPaused",
      "msg": "Program is paused — state-changing operations are frozen"
    },
    {
      "code": 6125,
      "name": "programNotPaused",
      "msg": "Program is not paused"
    },
    {
      "code": 6126,
      "name": "programAlreadyPaused",
      "msg": "Program is already paused"
    },
    {
      "code": 6127,
      "name": "guardianshipAlreadyActive",
      "msg": "A revocation request is already pending for this identity"
    },
    {
      "code": 6128,
      "name": "emptyClaimId",
      "msg": "Claim id cannot be all zeros"
    },
    {
      "code": 6129,
      "name": "invalidClaimType",
      "msg": "Invalid claim type"
    },
    {
      "code": 6130,
      "name": "emptyStatementHash",
      "msg": "Statement hash is required"
    },
    {
      "code": 6131,
      "name": "invalidClaimStatus",
      "msg": "Invalid claim status for this operation"
    },
    {
      "code": 6132,
      "name": "notClaimSubmitter",
      "msg": "Only the claim submitter can add evidence"
    },
    {
      "code": 6133,
      "name": "invalidEvidenceType",
      "msg": "Invalid evidence type"
    },
    {
      "code": 6134,
      "name": "emptyStorageReference",
      "msg": "Storage reference is required"
    },
    {
      "code": 6135,
      "name": "invalidConfidence",
      "msg": "Confidence must be between 0 and 100"
    },
    {
      "code": 6136,
      "name": "invalidAttestationResult",
      "msg": "Invalid attestation result"
    },
    {
      "code": 6137,
      "name": "quorumNotReached",
      "msg": "Attestation quorum not yet reached"
    },
    {
      "code": 6138,
      "name": "claimAlreadyVerified",
      "msg": "Claim already verified"
    },
    {
      "code": 6139,
      "name": "invalidRequiredAttestations",
      "msg": "Required attestations must be at least 1"
    },
    {
      "code": 6140,
      "name": "invalidSessionStatus",
      "msg": "Session is not in a valid state for this operation"
    },
    {
      "code": 6141,
      "name": "validatorNotActive",
      "msg": "Validator is not active"
    },
    {
      "code": 6142,
      "name": "challengeAlreadyResolved",
      "msg": "Challenge has already been resolved"
    },
    {
      "code": 6143,
      "name": "invalidQuorumConfig",
      "msg": "Quorum configuration is invalid"
    },
    {
      "code": 6144,
      "name": "validatorJailed",
      "msg": "Validator is jailed or slashed and cannot attest"
    },
    {
      "code": 6145,
      "name": "observerNotActive",
      "msg": "Observer is not active"
    },
    {
      "code": 6146,
      "name": "observerNotSuspended",
      "msg": "Observer is not suspended"
    },
    {
      "code": 6147,
      "name": "invalidGuardianType",
      "msg": "Invalid guardian type"
    },
    {
      "code": 6148,
      "name": "invalidGuardianClaimStatus",
      "msg": "Invalid guardian claim status"
    },
    {
      "code": 6149,
      "name": "invalidCrossBorderStatus",
      "msg": "Invalid cross-border verification status"
    },
    {
      "code": 6150,
      "name": "invalidAuditAction",
      "msg": "Invalid audit action"
    },
    {
      "code": 6151,
      "name": "invalidQuorumVoteChoice",
      "msg": "Invalid quorum vote choice"
    },
    {
      "code": 6152,
      "name": "quorumAlreadyResolved",
      "msg": "Quorum already resolved"
    },
    {
      "code": 6153,
      "name": "identityRightsAlreadyExists",
      "msg": "Identity rights account already exists for this identity/parcel/kind"
    },
    {
      "code": 6154,
      "name": "identityRightsNotFound",
      "msg": "Identity rights not found or inactive"
    },
    {
      "code": 6155,
      "name": "notDesignatedOffender",
      "msg": "Signer is not the designated offender"
    },
    {
      "code": 6156,
      "name": "wrongEndorsementAction",
      "msg": "Endorsement action does not match the requested governance operation"
    },
    {
      "code": 6157,
      "name": "missingReputation",
      "msg": "Validator reputation account must be provided for attestation"
    },
    {
      "code": 6158,
      "name": "attestationDigestMismatch",
      "msg": "Attestation signature_hash does not match the canonical digest over claim, validator, observation, result, and confidence"
    },
    {
      "code": 6159,
      "name": "duplicateValidator",
      "msg": "Duplicate validator in the declared validator set"
    },
    {
      "code": 6160,
      "name": "stringTooLong",
      "msg": "Profile note exceeds maximum length"
    },
    {
      "code": 6161,
      "name": "invalidProfileTier",
      "msg": "Invalid validator profile tier"
    },
    {
      "code": 6162,
      "name": "invalidPresenceProvenance",
      "msg": "Invalid presence provenance code"
    },
    {
      "code": 6163,
      "name": "invalidPresenceFix",
      "msg": "Presence fix is outside valid geographic bounds"
    },
    {
      "code": 6164,
      "name": "invalidAvailabilityStatus",
      "msg": "Invalid availability status"
    },
    {
      "code": 6165,
      "name": "invalidCapabilityCode",
      "msg": "Invalid capability code"
    },
    {
      "code": 6166,
      "name": "invalidCapabilityLevel",
      "msg": "Invalid capability level"
    },
    {
      "code": 6167,
      "name": "invalidEdgeType",
      "msg": "Invalid relationship edge type"
    },
    {
      "code": 6168,
      "name": "selfRelationshipEdge",
      "msg": "Self-relationship edges are not allowed"
    },
    {
      "code": 6169,
      "name": "invalidTaskClass",
      "msg": "Invalid task class"
    },
    {
      "code": 6170,
      "name": "invalidTaskStatus",
      "msg": "Invalid task status"
    },
    {
      "code": 6171,
      "name": "invalidTaskOutcome",
      "msg": "Invalid task outcome"
    },
    {
      "code": 6172,
      "name": "taskDeadlinePassed",
      "msg": "Task deadline has passed"
    },
    {
      "code": 6173,
      "name": "taskNotOpen",
      "msg": "Task is not open for assignment"
    },
    {
      "code": 6174,
      "name": "taskAlreadyFinalized",
      "msg": "Task is already finalized"
    },
    {
      "code": 6175,
      "name": "notTaskRequester",
      "msg": "Signer is not the task requester"
    },
    {
      "code": 6176,
      "name": "notTaskAssignee",
      "msg": "Signer is not an assigned validator for this task"
    },
    {
      "code": 6177,
      "name": "taskAlreadyAssigned",
      "msg": "Task already has the required number of assignments"
    },
    {
      "code": 6178,
      "name": "selfTaskAssignment",
      "msg": "Requester cannot be assigned to their own task"
    },
    {
      "code": 6179,
      "name": "requirementIndexMismatch",
      "msg": "Requirement index must append sequentially"
    },
    {
      "code": 6180,
      "name": "invalidTaskRequirement",
      "msg": "Invalid task requirement field or task id"
    },
    {
      "code": 6181,
      "name": "taskRequirementsFull",
      "msg": "Task has reached the maximum number of requirements"
    },
    {
      "code": 6182,
      "name": "invalidObservationSource",
      "msg": "Invalid observation source"
    },
    {
      "code": 6183,
      "name": "invalidObservationProvenance",
      "msg": "Invalid observation provenance"
    },
    {
      "code": 6184,
      "name": "invalidEvidenceArtifactKind",
      "msg": "Invalid evidence artifact kind"
    },
    {
      "code": 6185,
      "name": "evidenceIndexMismatch",
      "msg": "Evidence artifact index does not match manifest count"
    },
    {
      "code": 6186,
      "name": "evidenceManifestFull",
      "msg": "Evidence manifest has reached the maximum number of artifacts"
    },
    {
      "code": 6187,
      "name": "validatorNotEligible",
      "msg": "Validator failed multi-factor eligibility filters"
    },
    {
      "code": 6188,
      "name": "notRouteWinner",
      "msg": "Chosen candidate is not the randomized route winner"
    },
    {
      "code": 6189,
      "name": "tooManyRouteCandidates",
      "msg": "Too many route candidates or invalid competitor count"
    },
    {
      "code": 6190,
      "name": "routeAccountMismatch",
      "msg": "Route remaining_accounts do not match candidate list"
    },
    {
      "code": 6191,
      "name": "invalidFraudReason",
      "msg": "Invalid fraud reason code"
    },
    {
      "code": 6192,
      "name": "invalidFraudStatus",
      "msg": "Fraud status does not allow this transition"
    },
    {
      "code": 6193,
      "name": "selfFraudReport",
      "msg": "Reporters cannot file fraud against themselves"
    },
    {
      "code": 6194,
      "name": "committeeTooSmall",
      "msg": "Too few eligible well-reputed validators for a committee"
    },
    {
      "code": 6195,
      "name": "reviewAlreadyDecided",
      "msg": "Review already has a final decision"
    },
    {
      "code": 6196,
      "name": "notCommitteeMember",
      "msg": "Signer is not a member of this review committee"
    },
    {
      "code": 6197,
      "name": "duplicateCommitteeVote",
      "msg": "This committee member has already voted"
    },
    {
      "code": 6198,
      "name": "reviewNotFinalizable",
      "msg": "Not every committee member has voted yet"
    },
    {
      "code": 6199,
      "name": "invalidRestrictionStatus",
      "msg": "Invalid capability restriction status"
    },
    {
      "code": 6200,
      "name": "restrictionNotActive",
      "msg": "Restriction is not active"
    },
    {
      "code": 6201,
      "name": "appealWindowClosed",
      "msg": "Appeal window for this restriction has not opened"
    },
    {
      "code": 6202,
      "name": "notRestrictedWallet",
      "msg": "Signer is not the restricted wallet"
    },
    {
      "code": 6203,
      "name": "invalidAppealStatus",
      "msg": "Invalid fraud appeal status"
    },
    {
      "code": 6204,
      "name": "rehabTooEarly",
      "msg": "Rehabilitation period for this restriction has not elapsed"
    },
    {
      "code": 6205,
      "name": "capabilityRestricted",
      "msg": "Active capability restriction blocks this route"
    },
    {
      "code": 6206,
      "name": "invalidFeeBps",
      "msg": "Protocol fee exceeds the maximum allowed bps"
    },
    {
      "code": 6207,
      "name": "feePolicyNotInitialized",
      "msg": "Fee policy has not been initialized"
    },
    {
      "code": 6208,
      "name": "invalidCoverageParams",
      "msg": "Invalid coverage incentive parameters"
    },
    {
      "code": 6209,
      "name": "coverageIncentiveNotInitialized",
      "msg": "Coverage incentive has not been initialized"
    },
    {
      "code": 6210,
      "name": "quoteAlreadyExists",
      "msg": "A resource quote already exists for this task"
    },
    {
      "code": 6211,
      "name": "resourceQuoteMissing",
      "msg": "Resource quote is missing for this task"
    },
    {
      "code": 6212,
      "name": "escrowAlreadyFunded",
      "msg": "Task escrow is already funded"
    },
    {
      "code": 6213,
      "name": "taskNotCompleted",
      "msg": "Task is not completed"
    },
    {
      "code": 6214,
      "name": "assignmentNotSubmitted",
      "msg": "Validator has not submitted a result for this task"
    },
    {
      "code": 6215,
      "name": "escrowNotFunded",
      "msg": "Task escrow is not funded or already fully released"
    },
    {
      "code": 6216,
      "name": "rewardAlreadyClaimed",
      "msg": "This validator already claimed the task reward"
    },
    {
      "code": 6217,
      "name": "escrowNotRefundable",
      "msg": "Task is still active — escrow is not refundable"
    },
    {
      "code": 6218,
      "name": "claimWindowNotClosed",
      "msg": "Claim window for this completed task is still open"
    },
    {
      "code": 6219,
      "name": "nothingToRefund",
      "msg": "No refundable lamports remain in the escrow vault"
    },
    {
      "code": 6220,
      "name": "deviceMetadataTooLong",
      "msg": "Device metadata reference exceeds the maximum length"
    },
    {
      "code": 6221,
      "name": "invalidDeviceKey",
      "msg": "Device key must not be the zero key"
    },
    {
      "code": 6222,
      "name": "invalidDeviceStatus",
      "msg": "Invalid device status value"
    },
    {
      "code": 6223,
      "name": "deviceRevoked",
      "msg": "Device is revoked — this operation is no longer allowed"
    },
    {
      "code": 6224,
      "name": "notDeviceOwner",
      "msg": "Signer is not the device owner"
    },
    {
      "code": 6225,
      "name": "invalidJurisdictionPair",
      "msg": "Country pair is invalid (equal, reversed, or mismatched)"
    },
    {
      "code": 6226,
      "name": "invalidBindingStatus",
      "msg": "Invalid cross-border binding status value"
    },
    {
      "code": 6227,
      "name": "sameJurisdiction",
      "msg": "Verification must span two different jurisdictions"
    },
    {
      "code": 6228,
      "name": "jurisdictionMismatch",
      "msg": "Jurisdiction accounts or binding do not match the required pair"
    },
    {
      "code": 6229,
      "name": "invalidBindingExpiry",
      "msg": "Cross-border binding expiry must be in the future (or 0)"
    },
    {
      "code": 6230,
      "name": "undeclaredJurisdiction",
      "msg": "Validator profile or requirement jurisdiction is undeclared"
    },
    {
      "code": 6231,
      "name": "invalidSpatialDimension",
      "msg": "Spatial dimension must be 0 (2D), 1 (2.5D) or 2 (3D)"
    },
    {
      "code": 6232,
      "name": "invalidGeometrySource",
      "msg": "Unknown geometry source"
    },
    {
      "code": 6233,
      "name": "geometryVersionMismatch",
      "msg": "Geometry version index or parent asset does not match"
    },
    {
      "code": 6234,
      "name": "geometryVersionsFull",
      "msg": "Spatial asset has reached the maximum number of geometry versions"
    },
    {
      "code": 6235,
      "name": "invalidElevationRange",
      "msg": "Elevation range is invalid (min > max)"
    },
    {
      "code": 6236,
      "name": "geometryAlreadyVerified",
      "msg": "Geometry version is already verified"
    },
    {
      "code": 6237,
      "name": "invalidElevationSource",
      "msg": "Unknown elevation source, or provenance does not match the layer dimension"
    },
    {
      "code": 6238,
      "name": "geometryAlreadyAttested",
      "msg": "Validator has already attested this geometry version"
    },
    {
      "code": 6239,
      "name": "emptyEvidenceManifest",
      "msg": "Evidence manifest has no artifacts to anchor a version from"
    },
    {
      "code": 6240,
      "name": "geometryQuorumFull",
      "msg": "Geometry version quorum list has no free attestation slot"
    },
    {
      "code": 6241,
      "name": "invalidModelFormat",
      "msg": "Unknown 3D model format, or format is the UNKNOWN placeholder"
    },
    {
      "code": 6242,
      "name": "threeDModelsFull",
      "msg": "Spatial asset has reached the maximum number of 3D models"
    },
    {
      "code": 6243,
      "name": "modelGeometryVersionMismatch",
      "msg": "Model geometry version does not exist in the asset's version history"
    },
    {
      "code": 6244,
      "name": "threeDModelAlreadyVerified",
      "msg": "3D model is already verified"
    },
    {
      "code": 6245,
      "name": "spatialSnapshotsFull",
      "msg": "Spatial asset has reached the maximum number of snapshots"
    },
    {
      "code": 6246,
      "name": "invalidSnapshotLabel",
      "msg": "Snapshot label is too long"
    }
  ],
  "types": [
    {
      "name": "amalgamationRecord",
      "docs": [
        "One record per source parcel being merged into a result parcel.",
        "",
        "PDA seed: `[\"amalgamation\", result_parcel, source_parcel]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "resultParcel",
            "docs": [
              "The merged result parcel's PDA key."
            ],
            "type": "pubkey"
          },
          {
            "name": "sourceParcel",
            "docs": [
              "A source parcel being merged in."
            ],
            "type": "pubkey"
          },
          {
            "name": "sourceGeometryHash",
            "docs": [
              "Source's geometry hash at time of amalgamation."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "resultGeometryHash",
            "docs": [
              "Result's geometry hash."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "rightsMerged",
            "docs": [
              "Whether rights from this source have been merged."
            ],
            "type": "bool"
          },
          {
            "name": "initiatedBy",
            "docs": [
              "Wallet that initiated the amalgamation."
            ],
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "completedAt",
            "docs": [
              "When merge finished (0 if in progress)."
            ],
            "type": "i64"
          },
          {
            "name": "status",
            "docs": [
              "0=Pending, 1=Completed, 2=Failed."
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "appeal",
      "docs": [
        "Due-process appeal against an active CapabilityRestriction.",
        "PDA: `[\"appeal\", restriction, appellant, nonce]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "restriction",
            "type": "pubkey"
          },
          {
            "name": "appellant",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u16"
          },
          {
            "name": "evidenceHash",
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
            "name": "note",
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "decidedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "appealReviewOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "review",
            "type": "pubkey"
          },
          {
            "name": "appeal",
            "type": "pubkey"
          },
          {
            "name": "committeeSize",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "auditEntry",
      "docs": [
        "Append-only audit log of all verification state transitions.",
        "",
        "PDA: `[\"audit_entry\", entity, sequence_bytes]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "entity",
            "docs": [
              "The entity this audit entry is about (claim, session, challenge, etc.)."
            ],
            "type": "pubkey"
          },
          {
            "name": "sequence",
            "docs": [
              "Monotonic sequence number for this entity."
            ],
            "type": "u32"
          },
          {
            "name": "action",
            "docs": [
              "Type of transition (see audit_action constants)."
            ],
            "type": "u8"
          },
          {
            "name": "fromStatus",
            "docs": [
              "Previous status value."
            ],
            "type": "u8"
          },
          {
            "name": "toStatus",
            "docs": [
              "New status value."
            ],
            "type": "u8"
          },
          {
            "name": "actor",
            "docs": [
              "Who triggered the transition."
            ],
            "type": "pubkey"
          },
          {
            "name": "metadataHash",
            "docs": [
              "Optional metadata hash (sha-256 of additional context)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "auditEntryRecorded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "auditEntry",
            "type": "pubkey"
          },
          {
            "name": "entity",
            "type": "pubkey"
          },
          {
            "name": "sequence",
            "type": "u32"
          },
          {
            "name": "action",
            "type": "u8"
          },
          {
            "name": "fromStatus",
            "type": "u8"
          },
          {
            "name": "toStatus",
            "type": "u8"
          },
          {
            "name": "actor",
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "capabilityRestriction",
      "docs": [
        "Partial capability downgrade — NOT a ban, NOT a jail (Design Rule 13).",
        "PDA: `[\"capability_restriction\", wallet, capability_code]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
          },
          {
            "name": "report",
            "type": "pubkey"
          },
          {
            "name": "maxLevel",
            "type": "u8"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "appliedAt",
            "type": "i64"
          },
          {
            "name": "appealOpensAt",
            "type": "i64"
          },
          {
            "name": "rehabEligibleAt",
            "type": "i64"
          },
          {
            "name": "liftedAt",
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
      "name": "capabilityRestrictionApplied",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "restriction",
            "type": "pubkey"
          },
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
          },
          {
            "name": "maxLevel",
            "type": "u8"
          },
          {
            "name": "report",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "capabilityRestrictionLifted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "restriction",
            "type": "pubkey"
          },
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
          },
          {
            "name": "by",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "challenge",
      "docs": [
        "An on-chain challenge to a verified claim. Anyone may challenge a",
        "verified claim by providing evidence. Validators review and vote.",
        "",
        "PDA: `[\"challenge\", claim, challenger]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim being challenged."
            ],
            "type": "pubkey"
          },
          {
            "name": "challenger",
            "docs": [
              "Who filed the challenge."
            ],
            "type": "pubkey"
          },
          {
            "name": "challengeHash",
            "docs": [
              "SHA-256 of the off-chain challenge document / evidence."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "status",
            "docs": [
              "Current challenge status."
            ],
            "type": "u8"
          },
          {
            "name": "upholdVotes",
            "docs": [
              "Number of validators who voted to uphold."
            ],
            "type": "u8"
          },
          {
            "name": "overturnVotes",
            "docs": [
              "Number of validators who voted to overturn."
            ],
            "type": "u8"
          },
          {
            "name": "requiredVotes",
            "docs": [
              "Required votes to resolve."
            ],
            "type": "u8"
          },
          {
            "name": "voters",
            "docs": [
              "Validators who voted (deduplication record)."
            ],
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "reviewDeadline",
            "type": "i64"
          },
          {
            "name": "resolvedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "challengeFiled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "challenge",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "challenger",
            "type": "pubkey"
          },
          {
            "name": "challengeHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "reviewDeadline",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "challengeResolved",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "challenge",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "outcome",
            "type": "u8"
          },
          {
            "name": "resolvedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "challengeVoteRecorded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "challenge",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "voteUphold",
            "type": "bool"
          },
          {
            "name": "upholdVotes",
            "type": "u8"
          },
          {
            "name": "overturnVotes",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "claim",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claimId",
            "docs": [
              "Unique claim identifier (caller-provided, e.g. SHA-256 of the statement)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "parcel",
            "docs": [
              "The parcel this claim is about."
            ],
            "type": "pubkey"
          },
          {
            "name": "claimType",
            "docs": [
              "Type of claim (see claim_type constants)."
            ],
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "docs": [
              "Participant who submitted the claim."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "Current status (see claim_status constants)."
            ],
            "type": "u8"
          },
          {
            "name": "statementHash",
            "docs": [
              "sha-256 hash of the off-chain claim statement / document."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "version",
            "docs": [
              "Monotonic version counter. Incremented on status transitions."
            ],
            "type": "u8"
          },
          {
            "name": "evidenceCount",
            "docs": [
              "Number of evidence items attached to this claim."
            ],
            "type": "u8"
          },
          {
            "name": "observationCount",
            "docs": [
              "Number of observations submitted for this claim."
            ],
            "type": "u8"
          },
          {
            "name": "attestationCount",
            "docs": [
              "Number of attestations submitted for this claim."
            ],
            "type": "u8"
          },
          {
            "name": "requiredAttestations",
            "docs": [
              "Quorum threshold — number of attestations needed to verify."
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
          }
        ]
      }
    },
    {
      "name": "claimCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "claimId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "claimType",
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "type": "pubkey"
          },
          {
            "name": "statementHash",
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
      "name": "claimRejected",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "reason",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "claimSessionTracker",
      "docs": [
        "Singleton guard PDA that tracks which session is currently active for a",
        "given claim. Prevents opening multiple concurrent verification sessions",
        "for the same claim.",
        "",
        "PDA: `[\"claim_session_tracker\", claim_key]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim this tracker belongs to."
            ],
            "type": "pubkey"
          },
          {
            "name": "activeSession",
            "docs": [
              "The currently active session (zero key = no active session)."
            ],
            "type": "pubkey"
          },
          {
            "name": "bump",
            "docs": [
              "Bump seed for deterministic derivation."
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "claimVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "claimType",
            "type": "u8"
          },
          {
            "name": "attestationCount",
            "type": "u8"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "countryAllocated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "worldRegistry",
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "approvedAdmin",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "countryAllocation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "countryCode",
            "docs": [
              "ISO 3166-1 alpha-2 country code."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "approvedAdmin",
            "docs": [
              "Admin wallet approved to create this country's ValidatorRegistry."
            ],
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "coverageIncentive",
      "docs": [
        "Global coverage-incentive parameters.",
        "",
        "PDA: `[\"coverage_incentive\"]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "demandBps",
            "docs": [
              "Baseline demand contribution (bps)."
            ],
            "type": "u16"
          },
          {
            "name": "deficitBps",
            "docs": [
              "Coverage-deficit contribution (bps)."
            ],
            "type": "u16"
          },
          {
            "name": "difficultyBps",
            "docs": [
              "Base difficulty contribution; scaled ×required_validators (cap 4)."
            ],
            "type": "u16"
          },
          {
            "name": "strategicBps",
            "docs": [
              "Strategic-importance contribution for strategic task classes."
            ],
            "type": "u16"
          },
          {
            "name": "maxSubsidyBps",
            "docs": [
              "Hard cap applied after summing all contributions."
            ],
            "type": "u16"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "coverageIncentiveSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "demandBps",
            "type": "u16"
          },
          {
            "name": "deficitBps",
            "type": "u16"
          },
          {
            "name": "difficultyBps",
            "type": "u16"
          },
          {
            "name": "strategicBps",
            "type": "u16"
          },
          {
            "name": "maxSubsidyBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "credentialIssued",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "credentialHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "purpose",
            "type": "string"
          },
          {
            "name": "signerCount",
            "type": "u8"
          },
          {
            "name": "nullifierHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "issuedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "credentialNullifier",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "nullifierHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "credential",
            "type": "pubkey"
          },
          {
            "name": "consumedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "credentialRequest",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "requestHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "purpose",
            "type": "string"
          },
          {
            "name": "disclosureType",
            "type": "u8"
          },
          {
            "name": "regionRegistry",
            "type": "pubkey"
          },
          {
            "name": "signers",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "finalized",
            "type": "bool"
          },
          {
            "name": "credential",
            "type": {
              "option": "pubkey"
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
      "name": "credentialRequested",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "requestHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "purpose",
            "type": "string"
          },
          {
            "name": "regionRegistry",
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "credentialSigned",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "requestHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "signer",
            "type": "pubkey"
          },
          {
            "name": "signersCount",
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
      "name": "credentialVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "credentialHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "nullifierHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "purpose",
            "type": "string"
          },
          {
            "name": "disclosureType",
            "type": "u8"
          },
          {
            "name": "blockTime",
            "type": "i64"
          },
          {
            "name": "proofHash",
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
      "name": "crossBorderBinding",
      "docs": [
        "Jurisdiction-to-jurisdiction recognition link (RFC-012 §5 `CrossBorderBinding`).",
        "",
        "PDA: `[\"cross_border_binding\", country_min, country_max]` — the country pair",
        "is stored canonically ordered, so one PDA exists per unordered pair and",
        "routing can derive it from `TaskRequirement.jurisdiction` + profile",
        "jurisdiction without extra lookups."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "countryMin",
            "docs": [
              "Smaller of the two ISO 3166-1 alpha-2 codes (2 bytes)."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "countryMax",
            "docs": [
              "Larger of the two codes."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "jurisdictionMin",
            "docs": [
              "Jurisdiction PDA of `country_min`."
            ],
            "type": "pubkey"
          },
          {
            "name": "jurisdictionMax",
            "docs": [
              "Jurisdiction PDA of `country_max`."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "`binding_status` — ACTIVE / SUSPENDED / REVOKED (terminal)."
            ],
            "type": "u8"
          },
          {
            "name": "expiresAt",
            "docs": [
              "Unix expiry (0 = none)."
            ],
            "type": "i64"
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
      "name": "crossBorderBindingCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "binding",
            "type": "pubkey"
          },
          {
            "name": "countryMin",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "countryMax",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "expiresAt",
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
      "name": "crossBorderBindingStatusChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "binding",
            "type": "pubkey"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "updatedBy",
            "type": "pubkey"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderIdentityBound",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "binding",
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
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "boundAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderIdentityRebound",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "oldBinding",
            "type": "pubkey"
          },
          {
            "name": "newBinding",
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
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "boundAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderLinked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "crossBorderVerification",
            "type": "pubkey"
          },
          {
            "name": "binding",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "session",
            "type": "pubkey"
          },
          {
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "linkedBy",
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderRevoked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "crossBorderVerification",
            "type": "pubkey"
          },
          {
            "name": "binding",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "revokedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderSpanRecord",
      "docs": [
        "A verification that spanned two jurisdictions (RFC-012 §5",
        "`CrossBorderVerification`) — the auditable record that a validator from",
        "`country_home` produced a result for a task requiring `country_required`",
        "under an ACTIVE `CrossBorderBinding`.",
        "",
        "PDA: `[\"cross_border_verification\", task_id, validator]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
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
            "name": "reqIndex",
            "type": "u8"
          },
          {
            "name": "countryHome",
            "docs": [
              "The validator's declared `ValidatorProfile.jurisdiction`."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "countryRequired",
            "docs": [
              "The task requirement's jurisdiction."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "binding",
            "docs": [
              "The `CrossBorderBinding` PDA that authorized the span."
            ],
            "type": "pubkey"
          },
          {
            "name": "recordedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderVerification",
      "docs": [
        "Bridges cross-border identity bindings into the verification pipeline.",
        "",
        "PDA: `[\"cross_border_verification\", binding]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "binding",
            "docs": [
              "The JurisdictionBinding account being verified."
            ],
            "type": "pubkey"
          },
          {
            "name": "claim",
            "docs": [
              "Associated Claim (if any)."
            ],
            "type": "pubkey"
          },
          {
            "name": "session",
            "docs": [
              "Associated VerificationSession (if any)."
            ],
            "type": "pubkey"
          },
          {
            "name": "jurisdiction",
            "docs": [
              "The Jurisdiction account."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "Current status (0=pending, 1=verified, 2=revoked)."
            ],
            "type": "u8"
          },
          {
            "name": "verifiedBy",
            "docs": [
              "Who verified this binding."
            ],
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderVerificationRecorded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "verification",
            "type": "pubkey"
          },
          {
            "name": "taskId",
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
            "name": "countryHome",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "countryRequired",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "binding",
            "type": "pubkey"
          },
          {
            "name": "recordedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "crossBorderVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "crossBorderVerification",
            "type": "pubkey"
          },
          {
            "name": "binding",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "verifiedBy",
            "type": "pubkey"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deviceCalibrationSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "device",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "calibrationHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "calibratedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deviceIdentity",
      "docs": [
        "A registered capture device (phone, GNSS receiver, drone, survey",
        "instrument, 3D scanner, …) owned by a wallet.",
        "",
        "PDA: `[\"device_identity\", owner, device_nonce]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "Wallet that registered (and controls) the device."
            ],
            "type": "pubkey"
          },
          {
            "name": "deviceNonce",
            "docs": [
              "PDA seed companion — one device per (owner, nonce)."
            ],
            "type": "u16"
          },
          {
            "name": "deviceKey",
            "docs": [
              "The device's Ed25519 public key (signs observation envelopes)."
            ],
            "type": "pubkey"
          },
          {
            "name": "source",
            "docs": [
              "Phase 4 `observation_source` code the device captures as."
            ],
            "type": "u8"
          },
          {
            "name": "capabilities",
            "docs": [
              "Declared capability bitfield (`device_capability::*`)."
            ],
            "type": "u64"
          },
          {
            "name": "metadataRef",
            "docs": [
              "Model/firmware metadata reference (IPFS CID or URL)."
            ],
            "type": "string"
          },
          {
            "name": "calibrationHash",
            "docs": [
              "SHA-256 of the latest calibration certificate (all-zero = never)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "calibratedAt",
            "docs": [
              "When the calibration certificate was issued (0 = never)."
            ],
            "type": "i64"
          },
          {
            "name": "status",
            "docs": [
              "`device_status` — ACTIVE / SUSPENDED / REVOKED."
            ],
            "type": "u8"
          },
          {
            "name": "verified",
            "docs": [
              "A validator has verified this registration (claim ≠ fact until then)."
            ],
            "type": "bool"
          },
          {
            "name": "verifiedBy",
            "docs": [
              "Validator wallet that last verified."
            ],
            "type": "pubkey"
          },
          {
            "name": "verifyVersion",
            "docs": [
              "Monotonic counter, bumped on each verification."
            ],
            "type": "u32"
          },
          {
            "name": "registeredAt",
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
      "name": "deviceKeyRotated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "device",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "oldDeviceKey",
            "type": "pubkey"
          },
          {
            "name": "newDeviceKey",
            "type": "pubkey"
          },
          {
            "name": "rotatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deviceRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "device",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "deviceKey",
            "type": "pubkey"
          },
          {
            "name": "source",
            "type": "u8"
          },
          {
            "name": "registeredAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deviceStatusChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "device",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "oldStatus",
            "type": "u8"
          },
          {
            "name": "newStatus",
            "type": "u8"
          },
          {
            "name": "changedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deviceUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "device",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "capabilities",
            "type": "u64"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "deviceVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "device",
            "type": "pubkey"
          },
          {
            "name": "verifiedBy",
            "type": "pubkey"
          },
          {
            "name": "verifyVersion",
            "type": "u32"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "dispute",
      "docs": [
        "An on-chain dispute record bound to a parcel.",
        "",
        "PDA: `[\"dispute\", parcel, case_hash]`. The `case_hash` is a SHA-256 over",
        "the off-chain court document or complaint, anchoring evidence immutably."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "docs": [
              "The parcel under dispute."
            ],
            "type": "pubkey"
          },
          {
            "name": "filedBy",
            "docs": [
              "The wallet that filed the dispute."
            ],
            "type": "pubkey"
          },
          {
            "name": "caseHash",
            "docs": [
              "SHA-256 of the off-chain court document / complaint."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "status",
            "docs": [
              "Current dispute status (FILED → FROZEN → ADJUDICATED → EXECUTED)."
            ],
            "type": "u8"
          },
          {
            "name": "required",
            "docs": [
              "Required validator co-signatures to advance this dispute."
            ],
            "type": "u8"
          },
          {
            "name": "declaredCount",
            "docs": [
              "Number of validators declared at filing time (NOT actual signatures collected)."
            ],
            "type": "u8"
          },
          {
            "name": "validators",
            "docs": [
              "Declared validator set for this dispute."
            ],
            "type": {
              "array": [
                "pubkey",
                8
              ]
            }
          },
          {
            "name": "presentValidators",
            "docs": [
              "Validators that actually co-signed (recorded at freeze/adjudicate).",
              "Unlike `validators` (declared at filing), this is on-chain proof of",
              "who testified — events alone are not reliably indexable."
            ],
            "type": {
              "array": [
                "pubkey",
                8
              ]
            }
          },
          {
            "name": "presentCount",
            "docs": [
              "How many entries of `present_validators` are populated."
            ],
            "type": "u8"
          },
          {
            "name": "filedAt",
            "type": "i64"
          },
          {
            "name": "frozenAt",
            "type": "i64"
          },
          {
            "name": "adjudicatedAt",
            "type": "i64"
          },
          {
            "name": "outcome",
            "docs": [
              "Outcome of adjudication (OWNER_WINS or OWNER_LOSES). Set at adjudication."
            ],
            "type": "u8"
          },
          {
            "name": "newOwner",
            "docs": [
              "New owner if outcome is OWNER_LOSES. Set at adjudication."
            ],
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "disputeAdjudicated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "dispute",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "outcome",
            "type": "u8"
          },
          {
            "name": "newOwner",
            "type": "pubkey"
          },
          {
            "name": "adjudicatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "disputeCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "dispute",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "cancelledBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "disputeFiled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "dispute",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "filedBy",
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
            "name": "required",
            "type": "u8"
          },
          {
            "name": "declaredCount",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "emergencyInjection",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "docs": [
              "The ValidatorRegistry this injection targets."
            ],
            "type": "pubkey"
          },
          {
            "name": "requestedBy",
            "docs": [
              "Admin who queued the injection."
            ],
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "docs": [
              "Validator public key to inject."
            ],
            "type": "pubkey"
          },
          {
            "name": "executeAfter",
            "docs": [
              "Unix timestamp after which the injection can be executed."
            ],
            "type": "i64"
          },
          {
            "name": "executed",
            "docs": [
              "True once executed."
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "emergencyInjectionExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "type": "pubkey"
          },
          {
            "name": "executedBy",
            "type": "pubkey"
          },
          {
            "name": "newValidatorCount",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "emergencyInjectionQueued",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "type": "pubkey"
          },
          {
            "name": "requestedBy",
            "type": "pubkey"
          },
          {
            "name": "executeAfter",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "equivocationReported",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "reporter",
            "type": "pubkey"
          },
          {
            "name": "offender",
            "type": "pubkey"
          },
          {
            "name": "evidenceHash",
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
      "name": "escrowAccepted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "seller",
            "type": "pubkey"
          },
          {
            "name": "acceptedAt",
            "type": "i64"
          },
          {
            "name": "settleDeadline",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "escrowCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "cancelledBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "escrowCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "seller",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "escrowDeposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "depositAmount",
            "type": "u64"
          },
          {
            "name": "totalDeposited",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "escrowDisputed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "filer",
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
            "name": "dispute",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "escrowExpired",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "expiredAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "escrowRecord",
      "docs": [
        "An on-chain escrow record for a parcel sale.",
        "",
        "PDA seed: `[\"escrow\", parcel_key]`. One escrow per parcel."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "docs": [
              "The parcel being sold."
            ],
            "type": "pubkey"
          },
          {
            "name": "seller",
            "docs": [
              "Wallet of the seller (must control the parcel's ownership right)."
            ],
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "docs": [
              "Wallet of the buyer (set at creation)."
            ],
            "type": "pubkey"
          },
          {
            "name": "amount",
            "docs": [
              "Sale price in lamports."
            ],
            "type": "u64"
          },
          {
            "name": "depositAmount",
            "docs": [
              "Amount deposited so far by buyer."
            ],
            "type": "u64"
          },
          {
            "name": "vault",
            "docs": [
              "Vault PDA holding deposited SOL."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "Current escrow status."
            ],
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "depositedAt",
            "type": "i64"
          },
          {
            "name": "acceptedAt",
            "type": "i64"
          },
          {
            "name": "settleDeadline",
            "docs": [
              "accepted_at + SETTLEMENT_WINDOW_SECS"
            ],
            "type": "i64"
          },
          {
            "name": "cancelDeadline",
            "docs": [
              "created_at + CANCEL_WINDOW_SECS"
            ],
            "type": "i64"
          },
          {
            "name": "disputeCaseHash",
            "docs": [
              "Case hash if dispute filed (0 if none)."
            ],
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
      "name": "escrowSettled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "seller",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "settledAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "evidence",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim this evidence supports."
            ],
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "docs": [
              "Monotonic nonce within the claim (0, 1, 2, ...)."
            ],
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "docs": [
              "Participant who submitted this evidence."
            ],
            "type": "pubkey"
          },
          {
            "name": "evidenceType",
            "docs": [
              "Type of evidence (see evidence_type constants)."
            ],
            "type": "u8"
          },
          {
            "name": "contentHash",
            "docs": [
              "sha-256 hash of the off-chain evidence content."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "storageReference",
            "docs": [
              "Storage reference (IPFS CID, URL, or content address)."
            ],
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "observedAt",
            "docs": [
              "When the evidence was originally observed (off-chain), if applicable."
            ],
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "evidenceAdded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "evidence",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u8"
          },
          {
            "name": "evidenceType",
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "type": "pubkey"
          },
          {
            "name": "contentHash",
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
      "name": "evidenceArtifact",
      "docs": [
        "One artifact inside a manifest (photo / document / video / model / geometry).",
        "",
        "PDA: `[\"evidence_artifact\", manifest, artifact_index]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "manifest",
            "docs": [
              "Parent manifest (PDA seed)."
            ],
            "type": "pubkey"
          },
          {
            "name": "artifactIndex",
            "docs": [
              "Append-only index within the manifest (PDA seed)."
            ],
            "type": "u16"
          },
          {
            "name": "kind",
            "docs": [
              "One of `artifact_kind`."
            ],
            "type": "u8"
          },
          {
            "name": "source",
            "docs": [
              "Capture source — one of `observation_source` (reused for provenance richness)."
            ],
            "type": "u8"
          },
          {
            "name": "provenance",
            "docs": [
              "Provenance — one of `observation_provenance`."
            ],
            "type": "u8"
          },
          {
            "name": "contentHash",
            "docs": [
              "sha256 of the artifact bytes / content envelope."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "storageReference",
            "docs": [
              "IPFS CID, URL, or content address (≤128 bytes)."
            ],
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "evidenceArtifactAdded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "artifact",
            "type": "pubkey"
          },
          {
            "name": "manifest",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "artifactIndex",
            "type": "u16"
          },
          {
            "name": "kind",
            "type": "u8"
          },
          {
            "name": "source",
            "type": "u8"
          },
          {
            "name": "provenance",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "evidenceManifest",
      "docs": [
        "Structured list of evidence artifacts for a verification task.",
        "",
        "PDA: `[\"evidence_manifest\", task_id, nonce]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "docs": [
              "Task this manifest supports."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "submitter",
            "docs": [
              "Wallet that opened the manifest (PDA seed companion). Pays rent."
            ],
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "docs": [
              "Unique nonce per (task_id, submitter). PDA seed."
            ],
            "type": "u16"
          },
          {
            "name": "observation",
            "docs": [
              "Optional ObservationV2 this manifest expands (`Pubkey::default()` = none)."
            ],
            "type": "pubkey"
          },
          {
            "name": "artifactCount",
            "docs": [
              "Number of artifacts appended so far (0..=MAX_MANIFEST_ARTIFACTS)."
            ],
            "type": "u16"
          },
          {
            "name": "rootHash",
            "docs": [
              "Pre-declared or rolling manifest root hash (sha256 of artifact set).",
              "May be zero at open; clients recompute/verify off-chain."
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
          }
        ]
      }
    },
    {
      "name": "evidenceManifestSubmitted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "manifest",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "submitter",
            "type": "pubkey"
          },
          {
            "name": "observation",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "feePolicy",
      "docs": [
        "Global protocol fee rules.",
        "",
        "PDA: `[\"fee_policy\"]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "docs": [
              "Wallet allowed to update (registry admin during bootstrap)."
            ],
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "docs": [
              "Protocol fee in bps, charged on top of every task escrow funding."
            ],
            "type": "u16"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "feePolicySet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "fraudAppealDecided",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "appeal",
            "type": "pubkey"
          },
          {
            "name": "decision",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "fraudAppealFiled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "appeal",
            "type": "pubkey"
          },
          {
            "name": "restriction",
            "type": "pubkey"
          },
          {
            "name": "appellant",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "fraudReport",
      "docs": [
        "Accusation with evidence. Permissionless to file; never auto-punishes.",
        "PDA: `[\"fraud_report\", accused, reporter, nonce]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "accused",
            "type": "pubkey"
          },
          {
            "name": "reporter",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u16"
          },
          {
            "name": "evidenceHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "reasonCode",
            "type": "u8"
          },
          {
            "name": "note",
            "type": "string"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
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
      "name": "fraudReportSubmitted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "report",
            "type": "pubkey"
          },
          {
            "name": "accused",
            "type": "pubkey"
          },
          {
            "name": "reporter",
            "type": "pubkey"
          },
          {
            "name": "reasonCode",
            "type": "u8"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "fraudReviewFinalized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "review",
            "type": "pubkey"
          },
          {
            "name": "report",
            "type": "pubkey"
          },
          {
            "name": "decision",
            "type": "u8"
          },
          {
            "name": "votesUpheld",
            "type": "u8"
          },
          {
            "name": "votesDismissed",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "fraudReviewOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "review",
            "type": "pubkey"
          },
          {
            "name": "report",
            "type": "pubkey"
          },
          {
            "name": "accused",
            "type": "pubkey"
          },
          {
            "name": "committeeSize",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "fraudVoteCast",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "review",
            "type": "pubkey"
          },
          {
            "name": "voter",
            "type": "pubkey"
          },
          {
            "name": "uphold",
            "type": "bool"
          },
          {
            "name": "votesCast",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "genesisConfirmation",
      "docs": [
        "Record of a cross-country genesis confirmation."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "validator",
            "docs": [
              "Validator who confirmed."
            ],
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "docs": [
              "Country code of the confirming validator."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          }
        ]
      }
    },
    {
      "name": "genesisConfirmed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "worldRegistry",
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "confirmer",
            "type": "pubkey"
          },
          {
            "name": "confirmerCountry",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "confirmationsCount",
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
      "name": "genesisFinalized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "worldRegistry",
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "confirmations",
            "type": "u8"
          },
          {
            "name": "distinctCountries",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "genesisRequest",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "countryCode",
            "docs": [
              "The country being genesis'd."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "requestedBy",
            "docs": [
              "Admin requesting genesis (must match allocation)."
            ],
            "type": "pubkey"
          },
          {
            "name": "confirmations",
            "docs": [
              "Confirmations from validators of other countries."
            ],
            "type": {
              "vec": {
                "defined": {
                  "name": "genesisConfirmation"
                }
              }
            }
          },
          {
            "name": "finalized",
            "docs": [
              "True once quorum and diversity requirements are met."
            ],
            "type": "bool"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "genesisRequested",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "worldRegistry",
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "requestedBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "geometryVersion",
      "docs": [
        "One append-only entry in a spatial asset's geometry history",
        "(RFC-012 §5 `GeometryVersion`).",
        "",
        "PDA: `[\"geometry_version\", asset, version_le_bytes]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "asset",
            "docs": [
              "The owning `SpatialAsset` PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "docs": [
              "Redundant parcel key for cheap off-chain joins."
            ],
            "type": "pubkey"
          },
          {
            "name": "version",
            "docs": [
              "0-based append index."
            ],
            "type": "u32"
          },
          {
            "name": "geometryHash",
            "docs": [
              "SHA-256 canonical digest of the off-chain geometry document."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "source",
            "docs": [
              "`geometry_source` provenance code."
            ],
            "type": "u8"
          },
          {
            "name": "dimension",
            "docs": [
              "`spatial_dimension` of this observation layer."
            ],
            "type": "u8"
          },
          {
            "name": "storageReference",
            "docs": [
              "Off-chain pointer (PostGIS row / IPFS CID / storage URL), <= 128 chars."
            ],
            "type": "string"
          },
          {
            "name": "submittedBy",
            "docs": [
              "Whoever paid to anchor this claim (permissionless)."
            ],
            "type": "pubkey"
          },
          {
            "name": "submittedAt",
            "type": "i64"
          },
          {
            "name": "verified",
            "docs": [
              "Claim → fact: true once `attest_count >= required` validators attested."
            ],
            "type": "bool"
          },
          {
            "name": "verifiedBy",
            "docs": [
              "Validator whose attestation completed the quorum (zero until verified)."
            ],
            "type": "pubkey"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          },
          {
            "name": "evidenceManifest",
            "docs": [
              "EvidenceManifest this version was anchored from",
              "(`Pubkey::default()` = standalone permissionless append)."
            ],
            "type": "pubkey"
          },
          {
            "name": "elevationMinMm",
            "docs": [
              "This version's elevation envelope, millimetres (both 0 when",
              "`elevation_source == elevation_source::NONE`)."
            ],
            "type": "i32"
          },
          {
            "name": "elevationMaxMm",
            "type": "i32"
          },
          {
            "name": "elevationSource",
            "docs": [
              "`elevation_source` provenance code for the Z values above."
            ],
            "type": "u8"
          },
          {
            "name": "required",
            "docs": [
              "Quorum threshold resolved at append (global `QuorumConfig`, else 2;",
              "1..=MAX_VALIDATORS)."
            ],
            "type": "u8"
          },
          {
            "name": "attestors",
            "docs": [
              "Unique validator attestors so far (zero-filled padding after",
              "`attest_count`; append-only — slots are never reused)."
            ],
            "type": {
              "array": [
                "pubkey",
                8
              ]
            }
          },
          {
            "name": "attestCount",
            "docs": [
              "Confirmed attestations so far (0..=required until verified)."
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "geometryVersionAppended",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "geometryVersion",
            "type": "pubkey"
          },
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "version",
            "type": "u32"
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
            "name": "source",
            "type": "u8"
          },
          {
            "name": "dimension",
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "type": "pubkey"
          },
          {
            "name": "submittedAt",
            "type": "i64"
          },
          {
            "name": "evidenceManifest",
            "type": "pubkey"
          },
          {
            "name": "elevationMinMm",
            "type": "i32"
          },
          {
            "name": "elevationMaxMm",
            "type": "i32"
          },
          {
            "name": "elevationSource",
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
      "name": "geometryVersionAttested",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "geometryVersion",
            "type": "pubkey"
          },
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "attestCount",
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
      "name": "geometryVersionVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "geometryVersion",
            "type": "pubkey"
          },
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "verifiedBy",
            "type": "pubkey"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          },
          {
            "name": "attestCount",
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
      "name": "guardianClaim",
      "docs": [
        "Bridge between the guardian/court system and the verification pipeline.",
        "When a guardianship event occurs (e.g. court-ordered guardianship via",
        "`terra_identity`), a GuardianClaim can be created to link that event",
        "into the verification pipeline.",
        "",
        "PDA: `[\"guardian_claim\", claim]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The associated Claim account."
            ],
            "type": "pubkey"
          },
          {
            "name": "identity",
            "docs": [
              "The identity under guardianship."
            ],
            "type": "pubkey"
          },
          {
            "name": "triggeredBy",
            "docs": [
              "Who triggered the guardianship (court/guardian wallet)."
            ],
            "type": "pubkey"
          },
          {
            "name": "caseHash",
            "docs": [
              "SHA-256 of the court case document."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "guardianType",
            "docs": [
              "Guardian type (0=court, 1=council, 2=emergency)."
            ],
            "type": "u8"
          },
          {
            "name": "status",
            "docs": [
              "Current status (0=pending, 1=active, 2=resolved, 3=disputed)."
            ],
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "resolvedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "guardianClaimCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "guardianClaim",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "triggeredBy",
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
            "name": "guardianType",
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "guardianClaimDisputed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "guardianClaim",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "disputedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "guardianClaimResolved",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "guardianClaim",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "resolvedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "identityRightGranted",
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
            "name": "rightsKind",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "identityRightRevoked",
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
            "name": "rightsKind",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "identityRights",
      "docs": [
        "An identity-based right that links an Identity PDA to a Parcel.",
        "",
        "This is the foundation for replacing raw wallet ownership (`Parcel.owner`)",
        "with identity-centric ownership. Rather than a wallet pubkey holding a right,",
        "the Identity PDA is the holder — making rights portable across wallets.",
        "",
        "PDA: `[\"identity_rights\", identity, parcel, rights_kind]`.",
        "One IdentityRights per (identity, parcel, kind) is allowed."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "docs": [
              "The Identity PDA that holds this right."
            ],
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "docs": [
              "The parcel this right applies to."
            ],
            "type": "pubkey"
          },
          {
            "name": "rightsKind",
            "docs": [
              "Right kind (OWNERSHIP, USAGE, EASEMENT, etc.)."
            ],
            "type": "u8"
          },
          {
            "name": "granter",
            "docs": [
              "Who granted this right (must be the current owner or granter)."
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
            "name": "status",
            "docs": [
              "ACTIVE, EXPIRED, REVOKED."
            ],
            "type": "u8"
          },
          {
            "name": "notes",
            "type": "string"
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
      "name": "judgmentExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "dispute",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "outcome",
            "type": "u8"
          },
          {
            "name": "newOwner",
            "type": "pubkey"
          },
          {
            "name": "executedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "jurisdiction",
      "docs": [
        "A registered jurisdiction (country/zone) in the cross-border identity system.",
        "",
        "PDA seed: `[\"jurisdiction\", country_code]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "countryCode",
            "docs": [
              "ISO 3166-1 alpha-2 padded to 16 bytes (e.g. b\"KE\\x00...\" for Kenya)."
            ],
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "authority",
            "docs": [
              "The jurisdiction authority wallet (issues/revokes credentials)."
            ],
            "type": "pubkey"
          },
          {
            "name": "jurisdictionName",
            "docs": [
              "Human-readable name (max 64 chars)."
            ],
            "type": "string"
          },
          {
            "name": "credentialSchemaCid",
            "docs": [
              "IPFS CID of the W3C Verifiable Credential schema."
            ],
            "type": "string"
          },
          {
            "name": "revocationRegistry",
            "docs": [
              "On-chain or oracle reference for revocation checks."
            ],
            "type": "pubkey"
          },
          {
            "name": "verificationKeyHash",
            "docs": [
              "SHA-256 of the ZK verification key for this jurisdiction's circuit."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "algorithmId",
            "docs": [
              "0 = Groth16, 1 = FRI-STARK."
            ],
            "type": "u8"
          },
          {
            "name": "status",
            "docs": [
              "0 = Active, 1 = Suspended, 2 = Withdrawn."
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
          }
        ]
      }
    },
    {
      "name": "jurisdictionBinding",
      "docs": [
        "Binds a person (via identity_hash) to a jurisdiction with a ZK proof.",
        "",
        "PDA seed: `[\"cross_border_identity\", jurisdiction_key, identity_hash]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identityHash",
            "docs": [
              "The identity_hash from the existing Identity account."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "jurisdictionKey",
            "docs": [
              "The Jurisdiction PDA this binding belongs to."
            ],
            "type": "pubkey"
          },
          {
            "name": "credentialCommitment",
            "docs": [
              "Pedersen commitment to the credential."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "nullifier",
            "docs": [
              "Derived nullifier to prevent double-binding and proof reuse."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "proofData",
            "docs": [
              "The serialized ZK proof (max 512 bytes)."
            ],
            "type": "bytes"
          },
          {
            "name": "proofVersion",
            "docs": [
              "Version of the proof circuit (for future upgrades)."
            ],
            "type": "u8"
          },
          {
            "name": "algorithmId",
            "docs": [
              "0 = Groth16, 1 = FRI-STARK."
            ],
            "type": "u8"
          },
          {
            "name": "revoked",
            "docs": [
              "Whether this binding has been revoked."
            ],
            "type": "bool"
          },
          {
            "name": "revokedAt",
            "docs": [
              "Timestamp of revocation (0 if not revoked)."
            ],
            "type": "i64"
          },
          {
            "name": "revokedBy",
            "docs": [
              "Who revoked this binding."
            ],
            "type": "pubkey"
          },
          {
            "name": "boundAt",
            "docs": [
              "When the binding was created."
            ],
            "type": "i64"
          },
          {
            "name": "expiresAt",
            "docs": [
              "Optional expiry (0 = no expiry)."
            ],
            "type": "i64"
          },
          {
            "name": "verified",
            "docs": [
              "Whether a validator has attested this binding's proof off-chain.",
              "Binding is a permissionless *claim*; only verified bindings should be",
              "relied upon (circuit verification is deferred to audit; see RFC-006)."
            ],
            "type": "bool"
          },
          {
            "name": "verifiedBy",
            "docs": [
              "Validator that last verified this binding."
            ],
            "type": "pubkey"
          },
          {
            "name": "version",
            "docs": [
              "Monotonic counter, bumped on re-verification."
            ],
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "jurisdictionIdentityRevoked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "binding",
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
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "reason",
            "type": "string"
          },
          {
            "name": "revokedBy",
            "type": "pubkey"
          },
          {
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "jurisdictionMembershipVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "binding",
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
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "version",
            "type": "u32"
          },
          {
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "jurisdictionRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "type": {
              "array": [
                "u8",
                16
              ]
            }
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "algorithmId",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "jurisdictionUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "jurisdiction",
            "type": "pubkey"
          },
          {
            "name": "updatedBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "nominationConfirmed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "type": "pubkey"
          },
          {
            "name": "confirmer",
            "type": "pubkey"
          },
          {
            "name": "isPhysical",
            "type": "bool"
          },
          {
            "name": "confirmationsCount",
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
      "name": "nullifierRecord",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "nullifierHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "rootVersion",
            "type": "u32"
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "proofPurpose",
            "type": "string"
          },
          {
            "name": "disclosureType",
            "type": "u8"
          },
          {
            "name": "blockTime",
            "type": "i64"
          },
          {
            "name": "proofHash",
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
      "name": "observation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim this observation is about."
            ],
            "type": "pubkey"
          },
          {
            "name": "validator",
            "docs": [
              "Validator who made this observation."
            ],
            "type": "pubkey"
          },
          {
            "name": "observedAt",
            "docs": [
              "When the observation was made."
            ],
            "type": "i64"
          },
          {
            "name": "location",
            "docs": [
              "Location of observation (encoded as latitude/longitude i64 pair, or 0,0)."
            ],
            "type": {
              "array": [
                "i64",
                2
              ]
            }
          },
          {
            "name": "method",
            "docs": [
              "Method used for observation (e.g. 0=visual, 1=GPS, 2=survey, 3=document review)."
            ],
            "type": "u8"
          },
          {
            "name": "findingsHash",
            "docs": [
              "Free-form findings hash (sha-256 of detailed findings document)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "confidence",
            "docs": [
              "Confidence level (0-100)."
            ],
            "type": "u8"
          },
          {
            "name": "signatureHash",
            "docs": [
              "sha-256 of the signed observation payload (for signature verification)."
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
          }
        ]
      }
    },
    {
      "name": "observationSubmitted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "observation",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "method",
            "type": "u8"
          },
          {
            "name": "confidence",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "observationV2",
      "docs": [
        "Multi-source observation of a task subject.",
        "",
        "PDA: `[\"observation_v2\", task_id, observer, nonce]`",
        "",
        "Roles (Design Rule 7): `subject` ≠ `capture_device` ≠ `observer`",
        "(submitter). A document owned by Alice (`subject`), photographed on",
        "Bob's phone (`capture_device`), submitted by Charlie (`observer`)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "docs": [
              "Task this observation supports."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "observer",
            "docs": [
              "Submitting wallet (PDA seed). May differ from `capture_device`."
            ],
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "docs": [
              "Unique nonce per (task_id, observer). PDA seed."
            ],
            "type": "u16"
          },
          {
            "name": "subject",
            "docs": [
              "What is observed — claim, parcel, identity, or other account key."
            ],
            "type": "pubkey"
          },
          {
            "name": "captureDevice",
            "docs": [
              "Device that captured the raw signal / image (may equal observer)."
            ],
            "type": "pubkey"
          },
          {
            "name": "source",
            "docs": [
              "One of `observation_source`."
            ],
            "type": "u8"
          },
          {
            "name": "provenance",
            "docs": [
              "One of `observation_provenance`."
            ],
            "type": "u8"
          },
          {
            "name": "location",
            "docs": [
              "Lat/lon * 1e7; `[0,0]` = no location (document / remote)."
            ],
            "type": {
              "array": [
                "i64",
                2
              ]
            }
          },
          {
            "name": "observedAt",
            "docs": [
              "When the phenomenon was observed (device clock / capture time)."
            ],
            "type": "i64"
          },
          {
            "name": "findingsHash",
            "docs": [
              "sha256 of findings / structured observation payload."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "evidenceHash",
            "docs": [
              "sha256 of evidence pre-image (manifest root; Phase 5 expands)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "confidence",
            "docs": [
              "Confidence 0–100."
            ],
            "type": "u8"
          },
          {
            "name": "signatureHash",
            "docs": [
              "sha256 of signed observation envelope (device or submitter signature)."
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
          }
        ]
      }
    },
    {
      "name": "observationV2Submitted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "observation",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "observer",
            "type": "pubkey"
          },
          {
            "name": "subject",
            "type": "pubkey"
          },
          {
            "name": "captureDevice",
            "type": "pubkey"
          },
          {
            "name": "source",
            "type": "u8"
          },
          {
            "name": "provenance",
            "type": "u8"
          },
          {
            "name": "confidence",
            "type": "u8"
          },
          {
            "name": "nonce",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "observer",
      "docs": [
        "An on-chain observer (non-validator participant) who can submit",
        "observations for claims.",
        "",
        "PDA: `[\"observer\", wallet]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "docs": [
              "The observer's wallet pubkey."
            ],
            "type": "pubkey"
          },
          {
            "name": "identity",
            "docs": [
              "Optional linked identity (e.g. from terra_identity program)."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "Current status (ACTIVE, SUSPENDED)."
            ],
            "type": "u8"
          },
          {
            "name": "totalObservations",
            "docs": [
              "Total observations submitted."
            ],
            "type": "u64"
          },
          {
            "name": "confirmedObservations",
            "docs": [
              "Observations that were confirmed by quorum."
            ],
            "type": "u64"
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
      "name": "observerRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "observer",
            "type": "pubkey"
          },
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "registeredAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "observerStatusChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "observer",
            "type": "pubkey"
          },
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ownershipProofVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "nullifierHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "rootVersion",
            "type": "u32"
          },
          {
            "name": "proofPurpose",
            "type": "string"
          },
          {
            "name": "disclosureType",
            "type": "u8"
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "ownershipRoot",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "merkleRoot",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "version",
            "type": "u32"
          },
          {
            "name": "commitmentCount",
            "type": "u32"
          },
          {
            "name": "algorithmId",
            "type": "u8"
          },
          {
            "name": "snapshotCid",
            "type": "string"
          },
          {
            "name": "snapshotHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "authoritySignature",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          },
          {
            "name": "verificationKeyHash",
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
      "name": "ownershipRootUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "newMerkleRoot",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "version",
            "type": "u32"
          },
          {
            "name": "commitmentCount",
            "type": "u32"
          },
          {
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "parcel",
      "docs": [
        "A parcel of land.",
        "",
        "The parcel itself carries no ownership. Ownership/control is the canonical",
        "`Rights` PDA at `[\"ownership\", parcel]` with `rights_kind == OWNERSHIP`",
        "(see `ownership_pda()`); its `holder` may be a wallet or an Identity PDA",
        "(identity is an optional overlay, never a prerequisite). Authorization",
        "goes through `is_authorized_holder()` only."
      ],
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
              "Count of the parcel's non-ownership Rights PDAs (`[\"rights\", parcel,",
              "nonce]`): `grant_right` requires `nonce == rights_count` and",
              "increments it; `revoke_right` decrements it so a closed nonce can be",
              "reused by a later grant."
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
      "name": "parcelFrozen",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "dispute",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "frozenAt",
            "type": "i64"
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
            "name": "holder",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "parcelSubdivided",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "originalParcel",
            "type": "pubkey"
          },
          {
            "name": "subParcel",
            "type": "pubkey"
          },
          {
            "name": "newGeometryHash",
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
      "name": "parcelsAmalgamated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "resultParcel",
            "type": "pubkey"
          },
          {
            "name": "sourceParcel",
            "type": "pubkey"
          },
          {
            "name": "newGeometryHash",
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
          }
        ]
      }
    },
    {
      "name": "programPaused",
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
            "name": "pausedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "programUnpaused",
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
            "name": "unpausedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "proofVersionInvalidated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "staleVersion",
            "type": "u32"
          },
          {
            "name": "currentVersion",
            "type": "u32"
          },
          {
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "quorumConfig",
      "docs": [
        "On-chain quorum configuration. Defines how many attestations and what",
        "confidence level are required to verify a claim of a given type, in",
        "a given region. Global defaults can be overridden per-type or per-region.",
        "",
        "PDA: `[\"quorum_config\", parcel_type, region]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcelType",
            "docs": [
              "Parcel type this config applies to (0 = global default)."
            ],
            "type": "u8"
          },
          {
            "name": "region",
            "docs": [
              "Region this config applies to ([0,0] = global default)."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "requiredAttestations",
            "docs": [
              "Number of attestations required."
            ],
            "type": "u8"
          },
          {
            "name": "requiredConfidence",
            "docs": [
              "Minimum confidence level (0-100) required from each attestation."
            ],
            "type": "u8"
          },
          {
            "name": "setBy",
            "docs": [
              "Who set this config."
            ],
            "type": "pubkey"
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
      "name": "quorumConfigSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "config",
            "type": "pubkey"
          },
          {
            "name": "parcelType",
            "type": "u8"
          },
          {
            "name": "region",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "requiredAttestations",
            "type": "u8"
          },
          {
            "name": "requiredConfidence",
            "type": "u8"
          },
          {
            "name": "setBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "quorumReachabilityChecked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "totalValidators",
            "type": "u8"
          },
          {
            "name": "activeCount",
            "type": "u8"
          },
          {
            "name": "requiredEndorsements",
            "type": "u8"
          },
          {
            "name": "quorumReachable",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "quorumReached",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "confirmWeight",
            "type": "u16"
          },
          {
            "name": "quorumThreshold",
            "type": "u16"
          },
          {
            "name": "totalVotes",
            "type": "u8"
          },
          {
            "name": "resolvedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "quorumTally",
      "docs": [
        "Aggregated quorum tally for a claim.",
        "",
        "PDA: `[\"quorum_tally\", claim]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim this tally is for."
            ],
            "type": "pubkey"
          },
          {
            "name": "totalWeight",
            "docs": [
              "Total weight of all votes."
            ],
            "type": "u16"
          },
          {
            "name": "confirmWeight",
            "docs": [
              "Weight of confirm votes."
            ],
            "type": "u16"
          },
          {
            "name": "disputeWeight",
            "docs": [
              "Weight of dispute votes."
            ],
            "type": "u16"
          },
          {
            "name": "abstainWeight",
            "docs": [
              "Weight of abstain votes."
            ],
            "type": "u16"
          },
          {
            "name": "totalVotes",
            "docs": [
              "Total number of votes cast."
            ],
            "type": "u8"
          },
          {
            "name": "quorumThreshold",
            "docs": [
              "Required weight to pass quorum."
            ],
            "type": "u16"
          },
          {
            "name": "resolved",
            "docs": [
              "Whether quorum has been resolved."
            ],
            "type": "bool"
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
      "name": "quorumVote",
      "docs": [
        "A weighted vote cast by a validator on a claim.",
        "",
        "PDA: `[\"quorum_vote\", claim, voter]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim being voted on."
            ],
            "type": "pubkey"
          },
          {
            "name": "voter",
            "docs": [
              "The validator casting the vote."
            ],
            "type": "pubkey"
          },
          {
            "name": "vote",
            "docs": [
              "Vote choice (0=confirm, 1=dispute, 2=abstain)."
            ],
            "type": "u8"
          },
          {
            "name": "weight",
            "docs": [
              "Reputation-based weight at time of vote (min(reputation_score, 10000))."
            ],
            "type": "u16"
          },
          {
            "name": "stakeAmount",
            "docs": [
              "SOL staked at time of vote (if applicable, 0 otherwise)."
            ],
            "type": "u64"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "quorumVoteCast",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "voter",
            "type": "pubkey"
          },
          {
            "name": "vote",
            "type": "u8"
          },
          {
            "name": "weight",
            "type": "u16"
          },
          {
            "name": "confirmWeight",
            "type": "u16"
          },
          {
            "name": "disputeWeight",
            "type": "u16"
          },
          {
            "name": "totalVotes",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "registryCreated",
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
      "name": "reportDismissed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "reporter",
            "type": "pubkey"
          },
          {
            "name": "offender",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "reputationSlashed",
      "docs": [
        "Reputation slash (score hits 0). Distinct from staking `ValidatorSlashed`",
        "so Anchor IDL event discriminators stay unique."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "reputationScore",
            "type": "u16"
          },
          {
            "name": "slashedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "resourceQuote",
      "docs": [
        "Requester's estimated resource cost for a task (travel, materials, …).",
        "",
        "PDA: `[\"resource_quote\", task_id]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "task",
            "docs": [
              "Task account this quote belongs to."
            ],
            "type": "pubkey"
          },
          {
            "name": "quoter",
            "docs": [
              "Requester wallet that quoted."
            ],
            "type": "pubkey"
          },
          {
            "name": "resourceCost",
            "docs": [
              "Estimated resource cost in lamports (funded alongside the reward)."
            ],
            "type": "u64"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "reviewCase",
      "docs": [
        "Independent review of a FraudReport by a random well-reputed committee.",
        "PDA: `[\"review_case\", report]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "report",
            "type": "pubkey"
          },
          {
            "name": "accused",
            "type": "pubkey"
          },
          {
            "name": "committee",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "voted",
            "docs": [
              "Members who have already voted (prevents double-vote)."
            ],
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "votesCast",
            "type": "u8"
          },
          {
            "name": "votesUpheld",
            "type": "u8"
          },
          {
            "name": "votesDismissed",
            "type": "u8"
          },
          {
            "name": "decision",
            "type": "u8"
          },
          {
            "name": "openedAt",
            "type": "i64"
          },
          {
            "name": "decidedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "rewardAllocation",
      "docs": [
        "Distribution record for one validator's claim on a task escrow.",
        "",
        "PDA: `[\"reward_allocation\", task_id, validator]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "basePaid",
            "docs": [
              "Base share paid from the escrow vault."
            ],
            "type": "u64"
          },
          {
            "name": "subsidyPaid",
            "docs": [
              "Coverage subsidy paid from the treasury."
            ],
            "type": "u64"
          },
          {
            "name": "totalPaid",
            "type": "u64"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "rewardsClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "rewardsDistributed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "periodReward",
            "type": "u64"
          },
          {
            "name": "totalStaked",
            "type": "u64"
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
      "name": "rightRenewed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "oldRights",
            "type": "pubkey"
          },
          {
            "name": "holder",
            "type": "pubkey"
          },
          {
            "name": "granter",
            "type": "pubkey"
          },
          {
            "name": "oldExpiresAt",
            "type": "i64"
          },
          {
            "name": "newExpiresAt",
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
      "name": "rightStatusTransition",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "rights",
            "type": "pubkey"
          },
          {
            "name": "holder",
            "type": "pubkey"
          },
          {
            "name": "oldStatus",
            "type": "u8"
          },
          {
            "name": "newStatus",
            "type": "u8"
          },
          {
            "name": "expiresAt",
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
              "Party who granted the right (invariably the current holder of the",
              "canonical ownership right)."
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
          },
          {
            "name": "status",
            "docs": [
              "Time-bound credential status (RFC-009): ACTIVE, EXPIRING, EXPIRED, GRACE, RENEWED, REVOKED."
            ],
            "type": "u8"
          },
          {
            "name": "gracePeriodSecs",
            "docs": [
              "Grace period in seconds after expiry (0 = no grace). Set at grant time."
            ],
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "rightsMigrated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "oldParcel",
            "type": "pubkey"
          },
          {
            "name": "newParcel",
            "type": "pubkey"
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
            "name": "nonce",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "rightsMigrationComplete",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "oldParcel",
            "type": "pubkey"
          },
          {
            "name": "newParcel",
            "type": "pubkey"
          },
          {
            "name": "count",
            "type": "u8"
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
      "name": "slashingAppealed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "offender",
            "type": "pubkey"
          },
          {
            "name": "appealReason",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "slashingReport",
      "docs": [
        "One record per slashing report.",
        "",
        "PDA seed: `[\"slashing_report\", stake_pool_key, reporter_key, evidence_hash]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "docs": [
              "StakePool key."
            ],
            "type": "pubkey"
          },
          {
            "name": "reporter",
            "docs": [
              "Wallet that filed the report."
            ],
            "type": "pubkey"
          },
          {
            "name": "evidenceHash",
            "docs": [
              "SHA-256 of the evidence payload."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "offender",
            "docs": [
              "Validator accused of misbehavior."
            ],
            "type": "pubkey"
          },
          {
            "name": "offenseType",
            "docs": [
              "0=equivocation, 1=liveness, 2=collusion."
            ],
            "type": "u8"
          },
          {
            "name": "offenseDetails",
            "docs": [
              "Bounded details (e.g., two conflicting parcel hashes)."
            ],
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          },
          {
            "name": "reporterBond",
            "docs": [
              "SOL bonded by reporter (for false-report penalty)."
            ],
            "type": "u64"
          },
          {
            "name": "status",
            "docs": [
              "0=Pending, 1=Verified, 2=Slashed, 3=Appealed, 4=Rejected, 5=Dismissed."
            ],
            "type": "u8"
          },
          {
            "name": "filedAt",
            "docs": [
              "When the report was filed."
            ],
            "type": "i64"
          },
          {
            "name": "appealDeadline",
            "docs": [
              "filed_at + APPEAL_WINDOW_SECS."
            ],
            "type": "i64"
          },
          {
            "name": "resolvedAt",
            "docs": [
              "When the report was resolved (0 if pending)."
            ],
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "spatialAsset",
      "docs": [
        "The 3D/4D extension of a parcel (RFC-012 §5 `SpatialAsset`).",
        "",
        "PDA: `[\"spatial_asset\", parcel]` — exactly one per parcel."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "parcel",
            "docs": [
              "The Parcel PDA this asset extends."
            ],
            "type": "pubkey"
          },
          {
            "name": "authority",
            "docs": [
              "Registrar that initialized the asset (future lifecycle authority)."
            ],
            "type": "pubkey"
          },
          {
            "name": "dimensionality",
            "docs": [
              "Highest supported dimensionality: `spatial_dimension`."
            ],
            "type": "u8"
          },
          {
            "name": "elevationMinMm",
            "docs": [
              "Minimum elevation of the asset envelope, millimetres (may be < 0)."
            ],
            "type": "i32"
          },
          {
            "name": "elevationMaxMm",
            "docs": [
              "Maximum elevation of the asset envelope, millimetres (>= min)."
            ],
            "type": "i32"
          },
          {
            "name": "geometryVersionCount",
            "docs": [
              "Append-only cursor: next version index (0..=MAX_GEOMETRY_VERSIONS)."
            ],
            "type": "u32"
          },
          {
            "name": "latestGeometry",
            "docs": [
              "Geometry hash of the newest version (all-zero until the first append)."
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
      "name": "spatialAssetCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "dimensionality",
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "spatialSnapshot",
      "docs": [
        "A frozen copy of a spatial asset's indexes at one instant",
        "(RFC-012 §5 `SpatialSnapshot`).",
        "",
        "PDA: `[\"spatial_snapshot\", asset, snapshot_index_le_bytes]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "asset",
            "docs": [
              "The owning `SpatialAsset` PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "docs": [
              "Redundant parcel key for cheap off-chain joins."
            ],
            "type": "pubkey"
          },
          {
            "name": "snapshotIndex",
            "docs": [
              "0-based append index (from [`SpatialSnapshotMeta`])."
            ],
            "type": "u32"
          },
          {
            "name": "geometryVersionCount",
            "docs": [
              "Geometry cursor copied at creation: versions `[0, N)` existed then.",
              "Never changes afterwards (append-only history is safe to replay)."
            ],
            "type": "u32"
          },
          {
            "name": "latestGeometry",
            "docs": [
              "Latest geometry hash at snapshot time (all-zero if none yet)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "dimensionality",
            "docs": [
              "Asset dimensionality copied at creation (`spatial_dimension`)."
            ],
            "type": "u8"
          },
          {
            "name": "elevationMinMm",
            "docs": [
              "Asset elevation envelope copied at creation, millimetres."
            ],
            "type": "i32"
          },
          {
            "name": "elevationMaxMm",
            "type": "i32"
          },
          {
            "name": "label",
            "docs": [
              "Short human label for timelines (e.g. \"2022 baseline\"), <= 64 bytes."
            ],
            "type": "string"
          },
          {
            "name": "createdBy",
            "docs": [
              "Whoever paid to create this snapshot (permissionless)."
            ],
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "spatialSnapshotCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "snapshot",
            "type": "pubkey"
          },
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "snapshotIndex",
            "type": "u32"
          },
          {
            "name": "geometryVersionCount",
            "type": "u32"
          },
          {
            "name": "labelLen",
            "docs": [
              "UTF-8 byte length of the label (the event never carries the string",
              "itself — read it from the account)."
            ],
            "type": "u8"
          },
          {
            "name": "createdBy",
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "spatialSnapshotMeta",
      "docs": [
        "Append-only cursor for a spatial asset's snapshots.",
        "",
        "PDA: `[\"spatial_snapshot_meta\", asset]` — created lazily on the first",
        "snapshot (`init_if_needed`), so `SpatialAsset`'s deployed layout is",
        "untouched."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "asset",
            "docs": [
              "The owning `SpatialAsset` PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "count",
            "docs": [
              "Next snapshot index (0..=MAX_SPATIAL_SNAPSHOTS)."
            ],
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "stakeDeposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "stakePool",
      "docs": [
        "One stake pool per region, derived from the ValidatorRegistry key.",
        "",
        "PDA seed: `[\"stake_pool\", region_registry_key]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "regionRegistry",
            "docs": [
              "ValidatorRegistry key for this region."
            ],
            "type": "pubkey"
          },
          {
            "name": "totalStaked",
            "docs": [
              "Total SOL staked across all validators (lamports)."
            ],
            "type": "u64"
          },
          {
            "name": "rewardRateBps",
            "docs": [
              "Annual reward rate in basis points (e.g., 500 = 5%)."
            ],
            "type": "u16"
          },
          {
            "name": "accumulatedRewards",
            "docs": [
              "Total rewards accrued but not yet distributed (lamports)."
            ],
            "type": "u64"
          },
          {
            "name": "rewardPerTokenStored",
            "docs": [
              "Global reward-per-token accumulator, scaled by REWARD_PRECISION (1e9).",
              "When distribute_rewards fires: rpt += period_reward * PRECISION / total_staked.",
              "Validators claim: (rpt - stake.rpt_paid) * staked_amount / PRECISION."
            ],
            "type": "u64"
          },
          {
            "name": "lastRewardDistribution",
            "docs": [
              "Timestamp of last reward distribution."
            ],
            "type": "i64"
          },
          {
            "name": "slashCount",
            "docs": [
              "Total slashing events executed."
            ],
            "type": "u32"
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
      "name": "stakePoolCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "regionRegistry",
            "type": "pubkey"
          },
          {
            "name": "rewardRateBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "stakeWithdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "subdivisionRecord",
      "docs": [
        "One record per sub-parcel, linking it back to the original parcel.",
        "",
        "PDA seed: `[\"subdivision\", original_parcel, sub_parcel]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "originalParcel",
            "docs": [
              "The parent parcel's PDA key."
            ],
            "type": "pubkey"
          },
          {
            "name": "subParcel",
            "docs": [
              "The child sub-parcel's PDA key."
            ],
            "type": "pubkey"
          },
          {
            "name": "originalGeometryHash",
            "docs": [
              "Parent's geometry hash at time of subdivision."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "newGeometryHash",
            "docs": [
              "Child's geometry hash."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "surveyClaim",
            "docs": [
              "Verified SUBDIVISION claim PDA that recorded surveyor sign-off."
            ],
            "type": "pubkey"
          },
          {
            "name": "rightsMigrated",
            "docs": [
              "Whether rights have been migrated to the sub-parcel."
            ],
            "type": "bool"
          },
          {
            "name": "initiatedBy",
            "docs": [
              "Wallet that initiated the subdivision."
            ],
            "type": "pubkey"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "completedAt",
            "docs": [
              "When migration finished (0 if in progress)."
            ],
            "type": "i64"
          },
          {
            "name": "status",
            "docs": [
              "0=Pending, 1=Completed, 2=Failed."
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "successionParcelsTransferred",
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
            "name": "transferred",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "taskAssigned",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
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
            "name": "assignedBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "taskAssignment",
      "docs": [
        "Selected validator for a task (requester assignment or self-claim).",
        "",
        "PDA: `[\"task_assignment\", task_id, validator]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "validator",
            "docs": [
              "Assigned validator wallet."
            ],
            "type": "pubkey"
          },
          {
            "name": "requester",
            "docs": [
              "Requester wallet that created the task (audit trail)."
            ],
            "type": "pubkey"
          },
          {
            "name": "assignedBy",
            "docs": [
              "Who performed the assignment: requester = assign, validator = claim."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "One of `assignment_status`."
            ],
            "type": "u8"
          },
          {
            "name": "assignedAt",
            "type": "i64"
          },
          {
            "name": "submittedAt",
            "docs": [
              "0 until `submit_task_result`."
            ],
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
      "name": "taskCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "requester",
            "type": "pubkey"
          },
          {
            "name": "cancelledAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "taskEscrow",
      "docs": [
        "Locked funds for a verification task.",
        "",
        "PDA: `[\"task_escrow\", task_id]` — vault PDA: `[\"task_escrow_vault\", escrow]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "requester",
            "type": "pubkey"
          },
          {
            "name": "vault",
            "docs": [
              "Vault PDA holding the funded lamports."
            ],
            "type": "pubkey"
          },
          {
            "name": "amount",
            "docs": [
              "Total funded for distribution (reward + resource cost)."
            ],
            "type": "u64"
          },
          {
            "name": "feePaid",
            "docs": [
              "Protocol fee paid to the treasury at funding time."
            ],
            "type": "u64"
          },
          {
            "name": "released",
            "docs": [
              "Base rewards paid out so far."
            ],
            "type": "u64"
          },
          {
            "name": "releasedCount",
            "docs": [
              "How many validator shares have been claimed."
            ],
            "type": "u8"
          },
          {
            "name": "status",
            "docs": [
              "One of `task_escrow_status`."
            ],
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "fundedAt",
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
      "name": "taskEscrowFunded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "requester",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "feePaid",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "taskEscrowRefunded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "escrow",
            "type": "pubkey"
          },
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "requester",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "taskRequirement",
      "docs": [
        "One row of eligibility / evidence constraints for a task.",
        "",
        "PDA: `[\"task_requirement\", task_id, req_index]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "reqIndex",
            "docs": [
              "Append-only index (0, 1, … requirement_count-1)."
            ],
            "type": "u8"
          },
          {
            "name": "capabilityCode",
            "docs": [
              "Required capability (`capability_code`) or `CAPABILITY_ANY`."
            ],
            "type": "u8"
          },
          {
            "name": "minReputation",
            "docs": [
              "Minimum legacy `ValidatorReputation.reputation_score` (0 = none)."
            ],
            "type": "u16"
          },
          {
            "name": "minTier",
            "docs": [
              "Minimum `ValidatorProfile.tier` (0 = any)."
            ],
            "type": "u8"
          },
          {
            "name": "jurisdiction",
            "docs": [
              "ISO 3166-1 alpha-2 as two bytes; `[0,0]` = any jurisdiction."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "radiusM",
            "docs": [
              "Max distance from center in meters; 0 = no geographic constraint."
            ],
            "type": "u32"
          },
          {
            "name": "centerLatE7",
            "docs": [
              "Geographic center (degrees * 1e7) when `radius_m > 0`."
            ],
            "type": "i32"
          },
          {
            "name": "centerLonE7",
            "type": "i32"
          },
          {
            "name": "independenceBps",
            "docs": [
              "Preferred independence (relationship-graph) threshold in bps (0 = none)."
            ],
            "type": "u16"
          },
          {
            "name": "confidenceTargetBps",
            "docs": [
              "Minimum confidence the result should target (0–10000 bps)."
            ],
            "type": "u16"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "taskRequirementAdded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "reqIndex",
            "type": "u8"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
          },
          {
            "name": "minReputation",
            "type": "u16"
          },
          {
            "name": "minTier",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "taskResourcesQuoted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "quoter",
            "type": "pubkey"
          },
          {
            "name": "resourceCost",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "taskResultSubmitted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
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
            "name": "outcome",
            "type": "u8"
          },
          {
            "name": "resultCount",
            "type": "u8"
          },
          {
            "name": "completed",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "taskRewardClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "allocation",
            "type": "pubkey"
          },
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
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
            "name": "basePaid",
            "type": "u64"
          },
          {
            "name": "subsidyPaid",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "taskRouted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
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
            "name": "reqIndex",
            "type": "u8"
          },
          {
            "name": "eligibleCount",
            "type": "u16"
          },
          {
            "name": "competitorCount",
            "type": "u16"
          },
          {
            "name": "drawBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "threeDModel",
      "docs": [
        "One append-only entry in a spatial asset's 3D model history",
        "(RFC-012 §5 `ThreeDModel`).",
        "",
        "PDA: `[\"three_d_model\", asset, model_version_le_bytes]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "asset",
            "docs": [
              "The owning `SpatialAsset` PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "docs": [
              "Redundant parcel key for cheap off-chain joins."
            ],
            "type": "pubkey"
          },
          {
            "name": "modelVersion",
            "docs": [
              "0-based append index (from `ThreeDModelMeta`)."
            ],
            "type": "u32"
          },
          {
            "name": "geometryVersion",
            "docs": [
              "Geometry version this model was reconstructed from (provenance —",
              "always < `asset.geometry_version_count` at registration time)."
            ],
            "type": "u32"
          },
          {
            "name": "modelHash",
            "docs": [
              "SHA-256 canonical digest of the off-chain model bundle."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "storageReference",
            "docs": [
              "Off-chain pointer (tileset URL / IPFS CID / storage key), <= 128 chars."
            ],
            "type": "string"
          },
          {
            "name": "modelFormat",
            "docs": [
              "`model_format` code of the referenced bundle."
            ],
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "docs": [
              "Whoever paid to register this claim (permissionless)."
            ],
            "type": "pubkey"
          },
          {
            "name": "submittedAt",
            "type": "i64"
          },
          {
            "name": "verified",
            "docs": [
              "Claim → fact: true once a registered validator verified the hash."
            ],
            "type": "bool"
          },
          {
            "name": "verifiedBy",
            "docs": [
              "Validator who verified (zero until verified)."
            ],
            "type": "pubkey"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "threeDModelMeta",
      "docs": [
        "Append-only cursor for a spatial asset's 3D models.",
        "",
        "PDA: `[\"three_d_model_meta\", asset]` — created lazily on the first",
        "registration (`init_if_needed`), so `SpatialAsset`'s deployed layout is",
        "untouched."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "asset",
            "docs": [
              "The owning `SpatialAsset` PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "count",
            "docs": [
              "Next model version index (0..=MAX_3D_MODELS)."
            ],
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "threeDModelRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "model",
            "type": "pubkey"
          },
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "parcel",
            "type": "pubkey"
          },
          {
            "name": "modelVersion",
            "type": "u32"
          },
          {
            "name": "geometryVersion",
            "type": "u32"
          },
          {
            "name": "modelHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "modelFormat",
            "type": "u8"
          },
          {
            "name": "submittedBy",
            "type": "pubkey"
          },
          {
            "name": "submittedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "threeDModelVerified",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "model",
            "type": "pubkey"
          },
          {
            "name": "asset",
            "type": "pubkey"
          },
          {
            "name": "verifiedBy",
            "type": "pubkey"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "thresholdCredential",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "credentialHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "prover",
            "type": "pubkey"
          },
          {
            "name": "purpose",
            "type": "string"
          },
          {
            "name": "disclosureType",
            "type": "u8"
          },
          {
            "name": "regionRegistry",
            "type": "pubkey"
          },
          {
            "name": "aggregateSignature",
            "type": "bytes"
          },
          {
            "name": "signers",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "signerCount",
            "type": "u8"
          },
          {
            "name": "nullifierHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "consumed",
            "type": "bool"
          },
          {
            "name": "version",
            "type": "u32"
          },
          {
            "name": "issuedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "unbondingInitiated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "validatorActiveSet",
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
            "name": "isActive",
            "type": "bool"
          },
          {
            "name": "setBy",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "validatorActivityTracker",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "docs": [
              "The ValidatorRegistry this tracker belongs to."
            ],
            "type": "pubkey"
          },
          {
            "name": "validator",
            "docs": [
              "The validator whose activity is tracked."
            ],
            "type": "pubkey"
          },
          {
            "name": "lastActive",
            "docs": [
              "Unix timestamp of last recorded activity (heartbeat or signed tx)."
            ],
            "type": "i64"
          },
          {
            "name": "isActive",
            "docs": [
              "Soft flag — set to false by admin to forcibly exclude a validator from",
              "quorum without removing them from the registry."
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "validatorAdded",
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
            "name": "addedBy",
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
      "name": "validatorAvailability",
      "docs": [
        "Current availability state (distinct from reputation jail).",
        "",
        "PDA: `[\"validator_availability\", wallet]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "One of `availability_status`."
            ],
            "type": "u8"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorAvailabilityChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorCapability",
      "docs": [
        "Declared/verified capability for one capability code.",
        "",
        "PDA: `[\"validator_capability\", wallet, capability_code]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "capabilityCode",
            "docs": [
              "Capability code (`capability_code`)."
            ],
            "type": "u8"
          },
          {
            "name": "level",
            "docs": [
              "Confidence level (`capability_level`)."
            ],
            "type": "u8"
          },
          {
            "name": "evidenceHash",
            "docs": [
              "Optional evidence hash backing a VERIFIED/TRUSTED claim (0 = none)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "declaredAt",
            "type": "i64"
          },
          {
            "name": "verifiedAt",
            "docs": [
              "0 until verified; set when level reaches VERIFIED or above."
            ],
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
      "name": "validatorCapabilityDeclared",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "capabilityCode",
            "type": "u8"
          },
          {
            "name": "level",
            "type": "u8"
          },
          {
            "name": "verifiedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorEndorsed",
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
      "name": "validatorEndorsement",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "docs": [
              "The registry this endorsement applies to."
            ],
            "type": "pubkey"
          },
          {
            "name": "proposed",
            "docs": [
              "The validator pubkey being proposed (target of the action)."
            ],
            "type": "pubkey"
          },
          {
            "name": "action",
            "docs": [
              "P0-2: governance action this endorsement authorizes (ADD or REMOVE)."
            ],
            "type": "u8"
          },
          {
            "name": "endorsers",
            "docs": [
              "Validators who endorsed this proposal."
            ],
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "required",
            "docs": [
              "Required endorsements to approve."
            ],
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorJailed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "reputationScore",
            "type": "u16"
          },
          {
            "name": "jailedUntil",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorJurisdictionSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "jurisdiction",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorNominated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "sponsor",
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "type": "pubkey"
          },
          {
            "name": "assignedPhysicalConfirmer",
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          }
        ]
      }
    },
    {
      "name": "validatorNomination",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "docs": [
              "The registry this nomination applies to."
            ],
            "type": "pubkey"
          },
          {
            "name": "countryCode",
            "docs": [
              "Country code (ISO 3166-1 alpha-2)."
            ],
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "sponsor",
            "docs": [
              "Existing validator who nominated the candidate."
            ],
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "docs": [
              "Wallet being nominated for validator status."
            ],
            "type": "pubkey"
          },
          {
            "name": "documentsHash",
            "docs": [
              "SHA-256 of the candidate's identity documents."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "locationHash",
            "docs": [
              "SHA-256 of a witness location report (never raw GPS)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "assignedPhysicalConfirmer",
            "docs": [
              "Randomly selected physical confirmer (set at nomination time via",
              "deterministic seed, NOT sponsor's choice)."
            ],
            "type": "pubkey"
          },
          {
            "name": "confirmers",
            "docs": [
              "Validators who have confirmed (max NOMINATION_CONFIRMATIONS)."
            ],
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "finalized",
            "docs": [
              "True once 3 confirmations are collected and candidate is added."
            ],
            "type": "bool"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorNominationFinalized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "registry",
            "type": "pubkey"
          },
          {
            "name": "candidate",
            "type": "pubkey"
          },
          {
            "name": "confirmers",
            "type": {
              "vec": "pubkey"
            }
          }
        ]
      }
    },
    {
      "name": "validatorPresence",
      "docs": [
        "Dynamic geographic presence — timestamped, confidence-scored, expiring.",
        "",
        "Never treat as immutable identity. PDA: `[\"validator_presence\", wallet]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "latitudeE7",
            "docs": [
              "Latitude in degrees * 1e7 (i32) — compact fixed-point."
            ],
            "type": "i32"
          },
          {
            "name": "longitudeE7",
            "docs": [
              "Longitude in degrees * 1e7."
            ],
            "type": "i32"
          },
          {
            "name": "accuracyM",
            "docs": [
              "Horizontal accuracy in meters."
            ],
            "type": "u16"
          },
          {
            "name": "provenance",
            "docs": [
              "How the fix was obtained (`presence_provenance`)."
            ],
            "type": "u8"
          },
          {
            "name": "confidenceBps",
            "docs": [
              "Confidence in this presence fix (0–10000 bps)."
            ],
            "type": "u16"
          },
          {
            "name": "observedAt",
            "docs": [
              "When this fix was recorded."
            ],
            "type": "i64"
          },
          {
            "name": "expiresAt",
            "docs": [
              "After this time the presence is stale (observed_at + TTL)."
            ],
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
      "name": "validatorPresenceUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "provenance",
            "type": "u8"
          },
          {
            "name": "confidenceBps",
            "type": "u16"
          },
          {
            "name": "expiresAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorProfile",
      "docs": [
        "Core validator profile — identity + tier attributes (not location-bound).",
        "",
        "PDA: `[\"validator_profile\", wallet]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "docs": [
              "The validator's wallet (mobile participant — NOT a region)."
            ],
            "type": "pubkey"
          },
          {
            "name": "tier",
            "docs": [
              "Trust tier (`profile_tier`). Starts NEW; admin/governance promotes."
            ],
            "type": "u8"
          },
          {
            "name": "identityHash",
            "docs": [
              "Optional link to an off-chain/on-chain identity hash (0 = unset)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "note",
            "docs": [
              "Free-form operator note (device class, org, …) — not used for selection."
            ],
            "type": "string"
          },
          {
            "name": "jurisdiction",
            "docs": [
              "ISO 3166-1 alpha-2 home/operating jurisdiction (RFC-012 Phase 10);",
              "`[0,0]` = undeclared. Self-declared claim — routing treats it as the",
              "validator's country for jurisdiction-scoped tasks and cross-border",
              "binding checks; fraud governance handles abuse."
            ],
            "type": {
              "array": [
                "u8",
                2
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
      "name": "validatorProfileInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "tier",
            "type": "u8"
          },
          {
            "name": "initializedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorProfileTierChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "oldTier",
            "type": "u8"
          },
          {
            "name": "newTier",
            "type": "u8"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorProfileUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "tier",
            "type": "u8"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorRegistry",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "docs": [
              "Bootstrap admin who can add validators unilaterally in bootstrap mode."
            ],
            "type": "pubkey"
          },
          {
            "name": "validators",
            "docs": [
              "Current list of registered validators."
            ],
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "requiredEndorsements",
            "docs": [
              "Minimum endorsements needed for new additions in peer-consensus mode.",
              "Recomputed on every validator count change; not read from storage for",
              "authorization decisions."
            ],
            "type": "u8"
          },
          {
            "name": "paused",
            "docs": [
              "Emergency pause flag. When true, all pausable subsystem instructions",
              "(staking, cross-border, guardian, zk, escrow, dispute) reject",
              "state-changing calls. Core parcel/identity operations authorized by",
              "individual owners are unaffected — the admin stops processing via",
              "off-chain policy."
            ],
            "type": "bool"
          },
          {
            "name": "version",
            "docs": [
              "Monotonic counter bumped on each change."
            ],
            "type": "u32"
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
      "name": "validatorRelationshipEdge",
      "docs": [
        "Directed relationship graph edge (endorsement / co-validation / infra).",
        "",
        "Correlation ≠ fraud — used for review, not auto-punishment (RFC-012 §6.12).",
        "PDA: `[\"validator_edge\", from, to, edge_type]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "from",
            "type": "pubkey"
          },
          {
            "name": "to",
            "type": "pubkey"
          },
          {
            "name": "edgeType",
            "docs": [
              "One of `relationship_edge_type`."
            ],
            "type": "u8"
          },
          {
            "name": "weightBps",
            "docs": [
              "Optional strength (0–10000 bps); default 10000 for binary endorsement."
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
      "name": "validatorRelationshipEdgeCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "from",
            "type": "pubkey"
          },
          {
            "name": "to",
            "type": "pubkey"
          },
          {
            "name": "edgeType",
            "type": "u8"
          },
          {
            "name": "weightBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "validatorRemoved",
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
    },
    {
      "name": "validatorReputation",
      "docs": [
        "Tracks an individual validator's reputation and performance metrics.",
        "",
        "PDA: `[\"validator_reputation\", validator]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "validator",
            "docs": [
              "The validator's pubkey."
            ],
            "type": "pubkey"
          },
          {
            "name": "status",
            "docs": [
              "Current status (ACTIVE, JAILED, SLASHED)."
            ],
            "type": "u8"
          },
          {
            "name": "totalAttestations",
            "docs": [
              "Total attestations submitted."
            ],
            "type": "u64"
          },
          {
            "name": "confirmedAttestations",
            "docs": [
              "Attestations that were confirmed by quorum."
            ],
            "type": "u64"
          },
          {
            "name": "disputedAttestations",
            "docs": [
              "Attestations that were disputed/challenged."
            ],
            "type": "u64"
          },
          {
            "name": "reputationScore",
            "docs": [
              "Reputation score in basis points (0 = slashed, 10000 = perfect)."
            ],
            "type": "u16"
          },
          {
            "name": "jailedUntil",
            "docs": [
              "Timestamp when the validator was jailed (0 if not jailed)."
            ],
            "type": "i64"
          },
          {
            "name": "challengesReceived",
            "docs": [
              "Total number of successful challenges against this validator."
            ],
            "type": "u8"
          },
          {
            "name": "challengesRaised",
            "docs": [
              "Total number of challenges raised by this validator."
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
          }
        ]
      }
    },
    {
      "name": "validatorReputationInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "initializedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "validatorSlashed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "type": "pubkey"
          },
          {
            "name": "offender",
            "type": "pubkey"
          },
          {
            "name": "slashAmount",
            "type": "u64"
          },
          {
            "name": "slashBps",
            "type": "u16"
          },
          {
            "name": "reporter",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "validatorStake",
      "docs": [
        "One record per validator per region.",
        "",
        "PDA seed: `[\"validator_stake\", stake_pool_key, validator_key]`."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakePool",
            "docs": [
              "StakePool key."
            ],
            "type": "pubkey"
          },
          {
            "name": "validator",
            "docs": [
              "Validator's Ed25519 pubkey."
            ],
            "type": "pubkey"
          },
          {
            "name": "stakedAmount",
            "docs": [
              "Current stake in lamports."
            ],
            "type": "u64"
          },
          {
            "name": "unbondingAmount",
            "docs": [
              "Amount in unbonding period."
            ],
            "type": "u64"
          },
          {
            "name": "unbondingStartsAt",
            "docs": [
              "When unbonding began (0 if not unbonding)."
            ],
            "type": "i64"
          },
          {
            "name": "rewardPerTokenPaid",
            "docs": [
              "Snapshot of the pool's reward_per_token_stored at the time of last",
              "claim or deposit. Claimable = (pool.rpt - self.rpt_paid) * staked / PRECISION."
            ],
            "type": "u64"
          },
          {
            "name": "slashHistory",
            "docs": [
              "Number of past slashing events (for graduated severity)."
            ],
            "type": "u8"
          },
          {
            "name": "offenses",
            "docs": [
              "Recent offense flags: [equivocation, liveness, collusion, unused]."
            ],
            "type": {
              "array": [
                "u8",
                4
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
    },
    {
      "name": "verificationAttestation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim being attested."
            ],
            "type": "pubkey"
          },
          {
            "name": "validator",
            "docs": [
              "Validator submitting this attestation."
            ],
            "type": "pubkey"
          },
          {
            "name": "observation",
            "docs": [
              "The observation this attestation is based on."
            ],
            "type": "pubkey"
          },
          {
            "name": "result",
            "docs": [
              "Attestation result (confirmed, disputed, unable_to_verify)."
            ],
            "type": "u8"
          },
          {
            "name": "confidence",
            "docs": [
              "Confidence level (0-100)."
            ],
            "type": "u8"
          },
          {
            "name": "signatureHash",
            "docs": [
              "sha-256 of the signed attestation payload."
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
            "name": "protocolVersion",
            "docs": [
              "Protocol version for audit trail."
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "verificationAttestationSubmitted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "attestation",
            "type": "pubkey"
          },
          {
            "name": "validator",
            "type": "pubkey"
          },
          {
            "name": "result",
            "type": "u8"
          },
          {
            "name": "confidence",
            "type": "u8"
          },
          {
            "name": "attestationCount",
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
      "name": "verificationKeyUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "oldHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "newHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "updatedBy",
            "type": "pubkey"
          },
          {
            "name": "blockTime",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "verificationSession",
      "docs": [
        "A stateful session that tracks a claim through its full verification",
        "lifecycle. Links evidence, observations, and attestations. Manages",
        "timeout and quorum progression.",
        "",
        "PDA: `[\"verification_session\", claim, session_id]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claim",
            "docs": [
              "The claim being verified."
            ],
            "type": "pubkey"
          },
          {
            "name": "sessionId",
            "docs": [
              "Unique session identifier (caller-provided)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "status",
            "docs": [
              "Current session status."
            ],
            "type": "u8"
          },
          {
            "name": "openedBy",
            "docs": [
              "Participant who opened the session."
            ],
            "type": "pubkey"
          },
          {
            "name": "evidenceCount",
            "docs": [
              "Number of evidence items attached."
            ],
            "type": "u8"
          },
          {
            "name": "observationCount",
            "docs": [
              "Number of observations submitted."
            ],
            "type": "u8"
          },
          {
            "name": "attestationCount",
            "docs": [
              "Number of confirmatory attestations."
            ],
            "type": "u8"
          },
          {
            "name": "disputeCount",
            "docs": [
              "Number of disconfirmatory attestations."
            ],
            "type": "u8"
          },
          {
            "name": "requiredAttestations",
            "docs": [
              "Required attestations for quorum."
            ],
            "type": "u8"
          },
          {
            "name": "createdAt",
            "type": "i64"
          },
          {
            "name": "expiresAt",
            "type": "i64"
          },
          {
            "name": "closedAt",
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
      "name": "verificationSessionClosed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "session",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "status",
            "type": "u8"
          },
          {
            "name": "closedAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "verificationSessionOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "session",
            "type": "pubkey"
          },
          {
            "name": "claim",
            "type": "pubkey"
          },
          {
            "name": "sessionId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "openedBy",
            "type": "pubkey"
          },
          {
            "name": "expiresAt",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "verificationTask",
      "docs": [
        "First-class work unit (requester, subject, class, reward, deadline).",
        "",
        "PDA: `[\"task\", task_id]`"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "taskId",
            "docs": [
              "Client-provided unique id (e.g. SHA-256 of the task statement)."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "requester",
            "docs": [
              "Who created and funds the task (pays rent; Phase 8 escrow source)."
            ],
            "type": "pubkey"
          },
          {
            "name": "subject",
            "docs": [
              "What the task is about — claim, parcel, identity, or other account.",
              "Not constrained to one program (migration map: claims become subjects)."
            ],
            "type": "pubkey"
          },
          {
            "name": "taskClass",
            "docs": [
              "One of `task_class`."
            ],
            "type": "u8"
          },
          {
            "name": "status",
            "docs": [
              "One of `task_status`."
            ],
            "type": "u8"
          },
          {
            "name": "rewardLamports",
            "docs": [
              "Reward in lamports recorded for Phase 8 distribution (0 = unfunded)."
            ],
            "type": "u64"
          },
          {
            "name": "deadline",
            "docs": [
              "Unix ts after which the task cannot be claimed/assigned/submitted."
            ],
            "type": "i64"
          },
          {
            "name": "descriptionHash",
            "docs": [
              "sha256 of off-chain task description / requirements narrative."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "requirementCount",
            "docs": [
              "Number of `TaskRequirement` rows (append-only via req_index)."
            ],
            "type": "u8"
          },
          {
            "name": "assignedCount",
            "docs": [
              "Number of `TaskAssignment` rows created."
            ],
            "type": "u8"
          },
          {
            "name": "requiredValidators",
            "docs": [
              "How many submitted results are needed to complete (1..=MAX_TASK_VALIDATORS)."
            ],
            "type": "u8"
          },
          {
            "name": "resultCount",
            "docs": [
              "How many results have been submitted so far."
            ],
            "type": "u8"
          },
          {
            "name": "outcome",
            "docs": [
              "One of `task_outcome` (PENDING until first result)."
            ],
            "type": "u8"
          },
          {
            "name": "resultHash",
            "docs": [
              "Canonical hash of the aggregated result / report."
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
          },
          {
            "name": "completedAt",
            "docs": [
              "0 until COMPLETED."
            ],
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "verificationTaskCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "task",
            "type": "pubkey"
          },
          {
            "name": "taskId",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "requester",
            "type": "pubkey"
          },
          {
            "name": "subject",
            "type": "pubkey"
          },
          {
            "name": "taskClass",
            "type": "u8"
          },
          {
            "name": "deadline",
            "type": "i64"
          },
          {
            "name": "requiredValidators",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "worldRegistry",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "docs": [
              "Global founder — temporary sole authority during bootstrap phase."
            ],
            "type": "pubkey"
          },
          {
            "name": "allocations",
            "docs": [
              "Country allocations: which admin is approved to create a registry",
              "for which country code."
            ],
            "type": {
              "vec": {
                "defined": {
                  "name": "countryAllocation"
                }
              }
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
      "name": "worldRegistryCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "worldRegistry",
            "type": "pubkey"
          },
          {
            "name": "admin",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "zoneSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "zoneId",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "parcelCount",
            "type": "u32"
          },
          {
            "name": "currentRootVersion",
            "type": "u32"
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
      "name": "zoneSetRegistered",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "zoneSet",
            "type": "pubkey"
          },
          {
            "name": "zoneId",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "snapshotCid",
            "type": "string"
          },
          {
            "name": "createdAt",
            "type": "i64"
          }
        ]
      }
    }
  ]
};
