# Save file format (PES 2016 – eFootball PES 2021, and games built on it)

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

## Games built on PES 2021

The Football Life series (SmokePatch) ships a modified PES 2021 executable. Football Life
2025 kept the PES 2021 master key; **SP Football Life 2027** uses its own (in `crypto.rs`,
file header game version `SP Football Life 2027`, saves in
`Documents\SPGAME\Football Life\2027\save\`). The container is otherwise identical.

PES 2021 and the games built on it store their **shuffled master key** in bytes 0..64 of
the file header, so once a file header can be decrypted the master key is known.

## Unknown keys

When no known key opens a file, `Container::recover` (`crates/core/src/recover.rs`) finds
the keys from the file alone:

1. Each keystream word is `m[t+4] ^ ror(m[t+3],13) ^ ror(m[t+2],6) ^ rol(m[t+1],5) ^
   ror(m[t],10)`, where `m` are MT19937 outputs — a linear function of them over GF(2).
2. Master League data keeps large zero-filled areas (one of 168 KB right after the
   budgets in PES 2021-layout data). Guessing that a 2.6 KB window holds zeros (or
   0xFF bytes) makes the keystream there known; the four outputs before the window are
   128 unknown bits, and the MT19937 recurrence `y[n+624] = y[n+397] ^ twist(y[n],
   y[n+1])` on untempered outputs gives 640 linear equations. Gaussian elimination solves
   them or proves the guess wrong.
3. Running the recurrence backwards reaches the state `init_by_array` produced, which is
   inverted to read the 16-word block key. Every possible block start is tried; only the
   real one makes the 39 copies of the key that `init_by_array` mixes in agree.
4. The block key of `data.dat` is `rolling XOR u64(2)`. The rolling key decrypts the file
   header, which yields the master key (above). The file is only accepted if that master
   key opens it like any known key.

The search starts right after the Master League budgets, so it normally takes a few tens
of milliseconds; a full scan of a 20 MB file without a usable area takes a few seconds.

## Master League (inside `data.dat`)

| Field | Offset | Type |
|---|---|---|
| Transfer budget | `0xC7DD00` | u32, units of €100 |
| Salary budget | `0xC7DD14` | u32, units of €100 |
| Club followers | `0xB629A4` | u32 |
| Manager name | coach table (below) | 46 bytes, NUL-terminated UTF-8 (max 45) |

These offsets apply to PES 2021 (exe 1.01–1.04, data packs 1.0–4.0) and were also checked
on saves from "Football Life 2025" and "SP Football Life 2027" (whose `data.dat` is about
19 MB, against 13 MB, but keeps every field above at the same offset). Other versions may
move things around; the editor rejects saves whose `data.dat` is too small, but cannot
prove the offsets are right on untested versions — check the values shown before saving.

For games other than PES 2021 itself the editor also requires the coach table's first 100
records to hold their own slot numbers (`0, 1, 2, …`), which places the table — and with
it the layout — where PES 2021 keeps it.

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
