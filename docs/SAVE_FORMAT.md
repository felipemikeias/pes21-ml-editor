# Save file format (PES 2016 – eFootball PES 2021)

Based on the public-domain [pesXdecrypter](https://github.com/the4chancup/pesXdecrypter)
project and checked against real PES 2021 saves (decrypting and re-encrypting reproduces
the file byte for byte). Implementation:
[`crates/core/src/crypto.rs`](../crates/core/src/crypto.rs).

## Container

All integers are little-endian.

| Block | Size | Block key |
|---|---|---|
| Encryption header | 320 | `bytes[256..320] XOR shuffled(master_key)` |
| File header | 208 (PES 2018+) / 176 (2016–17) | `rolling XOR u64(208)` on each 8-byte lane |
| Description | `desc_size` | `rolling XOR u64(0)` |
| Logo (PNG) | `logo_size` | `rolling XOR u64(1)` |
| Data (`data.dat`) | `data_size` | `rolling XOR u64(2)` |
| Serial (UTF-16LE) | `serial_len × 2` | `rolling XOR u64(3)` |

* **shuffled(key)**: byte order reversed inside each 8-byte group.
* Bytes 256..320 of the encryption header are a random seed stored in clear text (the
  game picks a new one every time it saves).
* **rolling**: `header[0..64]` with `header[64..320]` XOR-folded in at `i & 63`.

### Stream cipher (`crypt_in_place`)

```text
mt = MT19937.init_by_array(64-byte key as 16 × u32)
c0..c3 = four outputs of mt
for each u32 word w:
    c4 = mt()
    w ^= c4 ^ c3 ^ c2 ^ c1 ^ c0
    c0, c1, c2, c3 = ror(c1,15), rol(c2,11), rol(c3,7), ror(c4,13)
tail (1–3 bytes): XOR with mt() ^ c3 ^ c2 ^ c1 ^ c0
```

XOR is symmetric: the same function encrypts and decrypts.

### File header (208 bytes)

| Offset | Size | Contents |
|---|---|---|
| 0 | 64 | "mystery data" (in PES 2021, the shuffled master key) |
| 64 | 4 | `data_size` |
| 68 | 4 | `logo_size` |
| 72 | 4 | `desc_size` |
| 76 | 4 | `serial_len` (UTF-16 code units) |
| 80 | 64 | hash (not checked by the game, according to pesXdecrypter) |
| 144 | 32 | file type, e.g. `ML`, `EDIT`, `SYSTEM` |
| 176 | 32 | game version, e.g. `eFootball PES 2021` (patches may change it) |

`320 + 208 + desc + logo + data + serial × 2` must equal the file size. The editor uses
this to reject corrupted files and to detect a wrong key.

## Master League (inside `data.dat`)

| Field | Offset | Type |
|---|---|---|
| Transfer budget | `0xC7DD00` | u32, units of €100 |
| Salary budget | `0xC7DD14` | u32, units of €100 |
| Club followers | `0xB629A4` | u32 |
| Manager name | coach table (below) | 46 bytes, NUL-terminated UTF-8 (max 45) |

These offsets apply to PES 2021 (exe 1.01–1.04, data packs 1.0–4.0) and were also checked
on a save from the community patch "Football Life 2025". Other versions may move things
around; the editor rejects saves whose `data.dat` is too small, but cannot prove the
offsets are right on untested versions — check the values shown before saving.

### Coach table and the player's manager

| Item | Value |
|---|---|
| Name of the coach in slot 0 | `0x133A38` |
| Record size | `0x258` (600 bytes) |
| Slots | 1300 (free slots have `slot = 0xFFFF`) |
| Record | `u32 slot`, `u32 coach_id`, name (46 bytes), 2 bytes of another field, … |
| Coach ID of the user-created manager | `0xFFFFFFF5` |

The player's manager has **no fixed slot**: database updates (official or from patches)
add coaches and push the record further down. In an unpatched save it is usually slot 661
(`0x1942C0`); in a "Football Life 2025" save it is slots 721 and 722, while slot 661
holds a real coach from the database. The editor therefore looks for records whose
`coach_id` is `0xFFFFFFF5` and whose `slot` matches their position. The game may keep more
than one record for the manager; all records with the same name as the first one are
renamed together. The two bytes after the name are never modified.
