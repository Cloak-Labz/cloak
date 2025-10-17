# Cloak - Visual System Flow

## 🎨 Simplified Architecture Diagram

```
                                 ┌─────────────────────────┐
                                 │     USER (CLIENT)       │
                                 │   - Wallet (Phantom)    │
                                 │   - Web Browser         │
                                 └───────┬─────────────────┘
                                         │
                         ┌───────────────┴────────────────┐
                         │                                │
                    DEPOSIT                          WITHDRAW
                         │                                │
                         ▼                                ▼
        ┌────────────────────────────┐    ┌──────────────────────────────┐
        │  1. Create Commitment      │    │  1. Scan Encrypted Notes     │
        │     C = H(amt,r,pk)        │    │  2. Get Merkle Proof         │
        │  2. Encrypt Output         │    │  3. Generate ZK Proof        │
        │  3. Send to Solana         │    │  4. Submit via Relay         │
        └──────────┬─────────────────┘    └─────────┬────────────────────┘
                   │                                  │
                   │ Transaction                      │ Proof + Outputs
                   ▼                                  ▼
        ┌────────────────────────────────────────────────────────────────┐
        │                    SOLANA BLOCKCHAIN                           │
        │         ┌──────────────────────────────────────┐              │
        │         │  SHIELD-POOL PROGRAM                 │              │
        │         │  c1oak6tetx...                       │              │
        │         │                                      │              │
        │         │  ┌──────────┐      ┌──────────┐     │              │
        │         │  │ DEPOSIT  │      │ WITHDRAW │     │              │
        │         │  │  (0% fee)│      │ (0.5% + │     │              │
        │         │  │          │      │  0.0025) │     │              │
        │         │  └────┬─────┘      └─────┬────┘     │              │
        │         └───────┼──────────────────┼──────────┘              │
        └─────────────────┼──────────────────┼─────────────────────────┘
                          │                  │
                Emit Event│                  │Verify Proof & Transfer
                          │                  │
                          ▼                  ▼
        ┌─────────────────────────┐  ┌──────────────────────┐
        │   INDEXER SERVICE       │  │  NULLIFIER SET       │
        │   - Watch Events        │  │  - Track Spent Notes │
        │   - Build Merkle Tree   │  │  - Prevent Doubles   │
        │   - Serve Proofs        │  │                      │
        │   - Store Commitments   │  └──────────────────────┘
        └───────────┬─────────────┘
                    │
                    │ Store & Update
                    ▼
        ┌─────────────────────────┐
        │   MERKLE TREE           │
        │   31 levels             │
        │   BLAKE3-256            │
        │   2^31 capacity         │
        │                         │
        │   C₁ C₂ C₃ C₄ ...      │
        └─────────────────────────┘
```

---

## 🔄 Deposit Flow (Step by Step)

```
USER                    SOLANA              INDEXER
 │                        │                    │
 │ 1. Generate keys       │                    │
 │    sk, r, pk           │                    │
 │                        │                    │
 │ 2. Compute C           │                    │
 │    C = H(amt,r,pk)     │                    │
 │                        │                    │
 │ 3. Encrypt note        │                    │
 │    enc = E(pk, note)   │                    │
 │                        │                    │
 │ 4. Submit tx ─────────>│                    │
 │    (SOL + C + enc)     │                    │
 │                        │                    │
 │                        │ 5. Validate        │
 │                        │    Transfer SOL    │
 │                        │    Store C         │
 │                        │                    │
 │                        │ 6. Emit Event ────>│
 │                        │    (C, enc, time)  │
 │                        │                    │
 │                        │                    │ 7. Append to tree
 │                        │                    │    leaf[n] = C
 │                        │                    │
 │                        │                    │ 8. Compute root
 │                        │                    │    root = H(...)
 │                        │                    │
 │ 9. Query root <────────────────────────────│
 │    & proofs            │                    │
 │                        │                    │
```

---

## 🔐 Withdraw Flow (Step by Step)

```
USER              INDEXER         SP1-PROVER      RELAY         SOLANA
 │                   │                │             │              │
 │ 1. Scan notes     │                │             │              │
 │    Decrypt own    │                │             │              │
 │                   │                │             │              │
 │ 2. Get root ─────>│                │             │              │
 │<──────────────────│                │             │              │
 │    root=0xabc     │                │             │              │
 │                   │                │             │              │
 │ 3. Get proof ────>│                │             │              │
 │<──────────────────│                │             │              │
 │    merkle_path    │                │             │              │
 │                   │                │             │              │
 │ 4. Generate proof │                │             │              │
 │   (Private inputs)│                │             │              │
 │ ──────────────────────────────────>│             │              │
 │   - amount, r, sk                  │             │              │
 │   - leaf_index                     │             │              │
 │   - merkle_path                    │             │              │
 │                   │                │             │              │
 │                   │            5. Run circuit   │              │
 │                   │               - Verify path  │              │
 │                   │               - Compute nf   │              │
 │                   │               - Check cons   │              │
 │                   │                │             │              │
 │<───────────────────────────────────│             │              │
 │   proof (260B)    │                │             │              │
 │                   │                │             │              │
 │ 6. Submit ────────────────────────────────────────>│            │
 │   (proof, pub_in, │                │             │              │
 │    outputs)       │                │             │              │
 │                   │                │             │              │
 │                   │                │         7. Validate        │
 │                   │                │            Check nf        │
 │                   │                │            Build tx        │
 │                   │                │             │              │
 │                   │                │         8. Submit ────────>│
 │                   │                │             │              │
 │                   │                │             │          9. Verify
 │                   │                │             │             - Root
 │                   │                │             │             - Nf
 │                   │                │             │             - Proof
 │                   │                │             │             - Outputs
 │                   │                │             │
 │                   │                │             │         10. Execute
 │                   │                │             │             - Mark nf
 │                   │                │             │             - Transfer
 │                   │                │             │             - Pay fee
 │                   │                │             │              │
 │<──────────────────────────────────────────────────────────────────┤
 │   SOL received!   │                │             │              │
 │                   │                │             │              │
```

---

## 🔍 Zero-Knowledge Proof Circuit

```
┌────────────────────────────────────────────────────────────────┐
│                   SP1 ZK CIRCUIT                               │
│                                                                │
│  PRIVATE INPUTS (Witness):                                    │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ • amount: 1.0 SOL                                        │ │
│  │ • r: 0x7f3a... (32 bytes randomness)                     │ │
│  │ • sk_spend: 0x9b2c... (32 bytes secret)                  │ │
│  │ • leaf_index: 42                                         │ │
│  │ • merkle_path: [h₁, h₂, ..., h₃₁] + [0,1,0,...]        │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  PUBLIC INPUTS:                                               │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ • root: 0xabc... (32 bytes)                              │ │
│  │ • nf: 0xdef... (32 bytes nullifier)                      │ │
│  │ • outputs_hash: 0x123... (32 bytes)                      │ │
│  │ • amount: 1.0 SOL                                        │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  CONSTRAINTS:                                                 │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                                                          │ │
│  │  1️⃣  pk_spend = H(sk_spend)                             │ │
│  │      ├─ Proves knowledge of secret                      │ │
│  │      └─ Output: 0x4d8f...                                │ │
│  │                                                          │ │
│  │  2️⃣  C = H(amount || r || pk_spend)                     │ │
│  │      ├─ Reconstructs commitment                         │ │
│  │      └─ Output: 0x8a7c...                                │ │
│  │                                                          │ │
│  │  3️⃣  MerkleVerify(C, path) == root                      │ │
│  │      ├─ Hash(C, h₁) = n₁                                │ │
│  │      ├─ Hash(n₁, h₂) = n₂                               │ │
│  │      ├─ ... (31 levels)                                 │ │
│  │      └─ Hash(n₃₀, h₃₁) == root ✓                        │ │
│  │                                                          │ │
│  │  4️⃣  nf = H(sk_spend || leaf_index)                     │ │
│  │      ├─ Computes unique nullifier                       │ │
│  │      └─ Output: 0xdef... ✓                               │ │
│  │                                                          │ │
│  │  5️⃣  sum(outputs) + fee == amount                       │ │
│  │      ├─ fee = (1.0 × 0.005) + 0.0025 = 0.0075          │ │
│  │      ├─ outputs = [0.9925 SOL to Bob]                  │ │
│  │      └─ 0.9925 + 0.0075 = 1.0 ✓                         │ │
│  │                                                          │ │
│  │  6️⃣  H(serialize(outputs)) == outputs_hash              │ │
│  │      ├─ Serialize: Bob_addr || 992500000               │ │
│  │      └─ Hash == 0x123... ✓                              │ │
│  │                                                          │ │
│  │  ALL CONSTRAINTS SATISFIED ✓                            │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  OUTPUT:                                                      │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ Groth16 Proof: 260 bytes                                 │ │
│  │ [0x1a, 0x2b, 0x3c, ...]                                  │ │
│  │                                                          │ │
│  │ Verification: Anyone can verify proof proves:           │ │
│  │ "I know a valid note in the tree with this root,        │ │
│  │  here's its nullifier, and outputs are correct"         │ │
│  └──────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

---

## 🗂️ Data Structures

```
┌──────────────────────────────────────────────────────────────┐
│  NOTE (Private Data)                                         │
├──────────────────────────────────────────────────────────────┤
│  struct Note {                                               │
│      amount: u64,           // 1_000_000_000 (1 SOL)        │
│      r: [u8; 32],          // Random blinding factor        │
│      sk_spend: [u8; 32],   // Secret spending key           │
│      leaf_index: u32,      // Position in tree (42)         │
│  }                                                           │
│                                                              │
│  Stored encrypted on-chain, decrypted by recipient         │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  COMMITMENT (Public)                                         │
├──────────────────────────────────────────────────────────────┤
│  C = BLAKE3(amount || r || pk_spend)                        │
│    = BLAKE3(1.0 SOL || 0x7f3a... || 0x4d8f...)             │
│    = 0x8a7c...                                              │
│                                                              │
│  Stored in Merkle tree, visible on-chain                    │
│  Hides: amount, recipient, spending authority               │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  NULLIFIER (Prevents Double-Spend)                          │
├──────────────────────────────────────────────────────────────┤
│  nf = BLAKE3(sk_spend || leaf_index)                        │
│     = BLAKE3(0x9b2c... || 42)                               │
│     = 0xdef...                                              │
│                                                              │
│  Revealed when spending, stored on-chain                    │
│  Links: note → spent status                                 │
│  Hides: commitment, amount, original depositor              │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  MERKLE TREE (31 Levels)                                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Level 31 (Root):        [0xabc...]                         │
│                          /        \                          │
│  Level 30:         [0x123...]  [0x456...]                   │
│                    /      \      /      \                    │
│  Level 29:     [..]  [..]  [..]  [..]                       │
│                 ⋮     ⋮     ⋮     ⋮                         │
│  Level 0:    [C₁] [C₂] [C₃] [C₄] [C₅] ...                  │
│              (Commitments)                                   │
│                                                              │
│  Proof for C₃ (leaf_index=2):                               │
│  pathElements = [C₄, H(C₁,C₂), ..., h₃₁]                   │
│  pathIndices = [1, 0, 1, 0, ...]  (left=0, right=1)        │
└──────────────────────────────────────────────────────────────┘
```

---

## 💰 Fee Structure

```
┌────────────────────────────────────────────────────────────┐
│  DEPOSIT FEES                                              │
├────────────────────────────────────────────────────────────┤
│  Protocol Fee:    0% (FREE)                                │
│  Network Fee:     ~0.000005 SOL                            │
│  Total Cost:      ~0.000005 SOL                            │
│                                                            │
│  Example: Deposit 1.0 SOL                                 │
│  ├─ User pays: 1.000005 SOL                               │
│  └─ Pool receives: 1.0 SOL                                │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  WITHDRAWAL FEES                                           │
├────────────────────────────────────────────────────────────┤
│  Variable Fee:    0.5% of amount                           │
│  Fixed Fee:       0.0025 SOL                               │
│  Network Fee:     ~0.000005 SOL                            │
│                                                            │
│  Example: Withdraw 1.0 SOL                                │
│  ├─ Variable: 1.0 × 0.005 = 0.0050 SOL                    │
│  ├─ Fixed: 0.0025 SOL                                     │
│  ├─ Total protocol fee: 0.0075 SOL                        │
│  ├─ Network fee: 0.000005 SOL                             │
│  └─ Recipient gets: 0.992495 SOL                          │
│                                                            │
│  Fee Distribution:                                         │
│  └─ 100% to treasury (future: can split with relayers)   │
└────────────────────────────────────────────────────────────┘
```

---

## 🔒 Security Properties Summary

```
✅ PRIVACY
  ├─ Deposit address ≠ Withdraw address
  ├─ Amount hidden in commitment
  ├─ Recipient hidden in commitment
  └─ Timing can be decorrelated via delays

✅ SECURITY
  ├─ Double-spend prevented by nullifiers
  ├─ Counterfeit prevented by Merkle proofs
  ├─ Front-running prevented (nf tied to sk)
  └─ Conservation enforced in circuit

⚠️  KNOWN LIMITATIONS
  ├─ Output amounts visible on-chain (MVP)
  ├─ Timing analysis possible
  ├─ Requires trust in SP1 proving system
  └─ Admin role for root updates (can be decentralized)

🔧 FUTURE ENHANCEMENTS
  ├─ Fixed denominations for better privacy
  ├─ Stealth addresses
  ├─ Decentralized root updates
  └─ Multi-token support
```

---

## 📊 System Status

```
┌────────────────────────────────────────────────────────────┐
│  PRODUCTION STATUS                                         │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ✅ Solana Program          DEPLOYED                       │
│     └─ c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp       │
│                                                            │
│  ✅ SP1 Circuit             WORKING                        │
│     └─ Proof generation: 30-60s                           │
│                                                            │
│  ✅ Indexer Service         OPERATIONAL                    │
│     └─ Merkle tree + API endpoints                        │
│                                                            │
│  ✅ Relay Service           OPERATIONAL                    │
│     └─ Transaction submission + tracking                  │
│                                                            │
│  🚧 Web Frontend            IN DEVELOPMENT                 │
│     └─ UI + WASM prover integration                       │
│                                                            │
│  ✅ Complete Flow Test      PASSING                        │
│     └─ Localnet + Testnet                                 │
│                                                            │
└────────────────────────────────────────────────────────────┘
```


