/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/terra_identity_program.json`.
 */
export type TerraIdentityProgram = {
  "address": "HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p",
  "metadata": {
    "name": "terraIdentityProgram",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Terra Identity program — PEOPLE domain (identity, succession, guardianship)"
  },
  "instructions": [
    {
      "name": "bindIdentity",
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
      "name": "cancelSuccession",
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
      "name": "endorseSuccession",
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
      "name": "executeRevokeGuardianship",
      "discriminator": [
        119,
        219,
        6,
        116,
        49,
        3,
        167,
        203
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
          "name": "newOwner",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "requestCourtGuardianship",
      "discriminator": [
        161,
        23,
        136,
        8,
        139,
        232,
        131,
        106
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
          "name": "scopeNotes",
          "type": "string"
        }
      ]
    },
    {
      "name": "requestSuccession",
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
      "name": "revokeGuardianship",
      "discriminator": [
        168,
        87,
        239,
        39,
        65,
        0,
        155,
        201
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
          "name": "revoker",
          "signer": true
        },
        {
          "name": "newOwner"
        }
      ],
      "args": [
        {
          "name": "newOwner",
          "type": "pubkey"
        }
      ]
    }
  ],
  "accounts": [
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
    }
  ],
  "events": [
    {
      "name": "courtGuardianshipRequested",
      "discriminator": [
        140,
        90,
        60,
        187,
        33,
        145,
        99,
        93
      ]
    },
    {
      "name": "guardianshipRevocationRequested",
      "discriminator": [
        78,
        209,
        237,
        197,
        250,
        152,
        126,
        7
      ]
    },
    {
      "name": "guardianshipRevoked",
      "discriminator": [
        128,
        41,
        40,
        28,
        37,
        45,
        126,
        221
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
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "emptyIdentityHash",
      "msg": "Identity hash is required"
    },
    {
      "code": 6001,
      "name": "emptyRecovery",
      "msg": "Recovery wallet is required"
    },
    {
      "code": 6002,
      "name": "emptySuccessor",
      "msg": "Successor wallet is required"
    },
    {
      "code": 6003,
      "name": "invalidSuccessionKind",
      "msg": "Invalid succession kind"
    },
    {
      "code": 6004,
      "name": "successorIsOwner",
      "msg": "Successor must differ from the current owner"
    },
    {
      "code": 6005,
      "name": "notAuthorized",
      "msg": "Not authorized to perform this action"
    },
    {
      "code": 6006,
      "name": "notValidator",
      "msg": "Signing wallet is not a declared validator for this succession"
    },
    {
      "code": 6007,
      "name": "validatorOwnsAsset",
      "msg": "A validator cannot be the owner of the asset being validated"
    },
    {
      "code": 6008,
      "name": "successionAlreadyEffective",
      "msg": "Succession has already become effective"
    },
    {
      "code": 6009,
      "name": "validationLimitReached",
      "msg": "No more validators may endorse this succession (limit reached)"
    },
    {
      "code": 6010,
      "name": "notSuccessor",
      "msg": "Only the named successor may claim this succession"
    },
    {
      "code": 6011,
      "name": "successionNotYetEffective",
      "msg": "Succession is not yet effective"
    },
    {
      "code": 6012,
      "name": "insufficientValidations",
      "msg": "Succession requires validator endorsements before it can be claimed"
    },
    {
      "code": 6013,
      "name": "attestationMismatch",
      "msg": "Attestation does not belong to this parcel"
    },
    {
      "code": 6014,
      "name": "identityMismatch",
      "msg": "Identity owner does not match the parcel owner"
    },
    {
      "code": 6015,
      "name": "notOwner",
      "msg": "Only the current owner can perform this action"
    },
    {
      "code": 6016,
      "name": "invalidThreshold",
      "msg": "Required threshold exceeds the number of validators"
    },
    {
      "code": 6017,
      "name": "emptyCaseHash",
      "msg": "Court case hash is required"
    },
    {
      "code": 6018,
      "name": "notesTooLong",
      "msg": "Notes exceed the maximum length of 128"
    },
    {
      "code": 6019,
      "name": "guardianshipGraceTooShort",
      "msg": "Guardianship grace period is below the 90-day minimum"
    },
    {
      "code": 6020,
      "name": "guardianshipThresholdTooLow",
      "msg": "Guardianship requires at least 3 validator endorsements"
    },
    {
      "code": 6021,
      "name": "noProposalFound",
      "msg": "No proposal found"
    },
    {
      "code": 6022,
      "name": "guardianshipAlreadyActive",
      "msg": "A revocation request is already pending for this identity"
    },
    {
      "code": 6023,
      "name": "settlementNotYetEffective",
      "msg": "Settlement not yet effective"
    },
    {
      "code": 6024,
      "name": "parcelDataTooShort",
      "msg": "Parcel account data is too short for deserialization"
    },
    {
      "code": 6025,
      "name": "parcelDeserializeFailed",
      "msg": "Failed to deserialize parcel account data"
    },
    {
      "code": 6026,
      "name": "parcelOwnerMismatch",
      "msg": "Parcel owner does not match the identity owner"
    },
    {
      "code": 6027,
      "name": "alreadyEndorsed",
      "msg": "This validator has already endorsed this succession"
    },
    {
      "code": 6028,
      "name": "duplicateValidator",
      "msg": "Duplicate validator in the declared validator set"
    }
  ],
  "types": [
    {
      "name": "courtGuardianshipRequested",
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
            "name": "scopeNotes",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "guardianshipRevocationRequested",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "previousGuardian",
            "type": "pubkey"
          },
          {
            "name": "newOwner",
            "type": "pubkey"
          },
          {
            "name": "requestedBy",
            "type": "pubkey"
          },
          {
            "name": "revokeAfter",
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
      "name": "guardianshipRevoked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "identity",
            "type": "pubkey"
          },
          {
            "name": "previousGuardian",
            "type": "pubkey"
          },
          {
            "name": "newOwner",
            "type": "pubkey"
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
      "name": "identity",
      "type": {
        "kind": "struct",
        "fields": [
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
          },
          {
            "name": "parcelCount",
            "type": "u16"
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
            "name": "pendingRevocation",
            "type": "bool"
          },
          {
            "name": "pendingNewOwner",
            "type": "pubkey"
          },
          {
            "name": "revokeAfter",
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
      "name": "succession",
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
            "name": "requestedAt",
            "type": "i64"
          },
          {
            "name": "effectiveAt",
            "type": "i64"
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
            "name": "validationsCount",
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
          },
          {
            "name": "endorsers",
            "docs": [
              "Tracks which validators have endorsed. Each validator can endorse at most once."
            ],
            "type": {
              "array": [
                "pubkey",
                8
              ]
            }
          },
          {
            "name": "endorsersCount",
            "type": "u8"
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
    }
  ]
};
