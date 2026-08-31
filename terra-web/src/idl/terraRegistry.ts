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
    }
  ],
  "events": [
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
    }
  ],
  "types": [
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
    }
  ]
};
