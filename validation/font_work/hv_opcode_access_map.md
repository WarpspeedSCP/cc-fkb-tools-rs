# Verified opcode → heap access map (re-verified against the IDB)

Session 2026-XX, following `hv_engine_slots.md` / `hv_script_scan.py`. This supersedes the heap
parts of `opcode_reference.md` where they disagree; every row below was read out of `sub_40A050`
(or the pre-switch 0x01 handler) in the live IDB, not from `llm_work/`.

## Method

1. Case entry addresses for all 254 opcode bytes from the compaction table `byte_4108E0` +
   jump table `jpt_40A392` (`0x4106E4`); opcodes sharing entry `0x4103BD` are the unimplemented set.
2. For each implemented entry, walk the body linearly (bounded by the next entry / `jmp loc_40B4D*`
   epilogue) and find operands that index the array: `sjisText+0DDB7Ah[reg*2]` or
   `[reg*2+6D9148h]`. For each, walk backwards to find which operand load feeds the index register
   (`movsx/movzx reg, word ptr [ebp+K]` → operand at instruction offset `K+1`).
3. Fixed slots: `o_mem` operands with `addr` inside `0x6D9148..0x6D9918` (excluding the base
   `0x6D9148`, which is the indexed form). Indirection condition bytes were verified by
   disassembly at the sites.
4. Cross-checked against the corpus (`hv_script_scan.py`, 324 files, byte-exact rebuild,
   `problems=0`) and against `hv_gate_hist.py` (all indirection conditions are false in the
   corpus).

## Variable-index operands — verified (one table per opcode)

Every variable-index operand is a **little-endian `u16`** word in the instruction stream. `+N` is an
instruction-relative offset (opcode byte = `+0`); `t+N` is relative to a choice's trailer (the byte
after the choice string). In each table:

* **`access`** = what the opcode does to the slot: `r` reads it, `w` writes it, `rw` reads *and*
  writes it.
* **`condition`** = what makes the word a variable index; when it is false, the same word is a
  literal (coordinates, ids, etc.).
* **`verified at`** = the instruction address(es) in the IDB the row comes from.

**17 opcodes consume variable indices — the complete set** (30 slot operands; plus 0x0D's two
non-slot words). No other implemented case contains an `[reg*2+6D9148h]` access (mechanical scan of
all 126 distinct bodies).

### Overview

| opcode | index operands (instruction offset) | access | conditioned? |
|---|---|---|---|
| 0x01 | arg1 (+2), arg2 (+4) | r, r | yes |
| 0x02 | availability (t+1), second value (t+8) — per choice | r, r | yes |
| 0x03 | destination (+2), source (+5) | w/rw, r | source only |
| 0x0C | destination (+1) | w | no |
| 0x0D | start of range (+1) | w | no |
| 0x23, 0x27 | position word 1 (+2), position word 2 (+4) | r, r | yes |
| 0x33 | minutes (+1), seconds (+3), milliseconds (+5) | w, w, w | no |
| 0x43 | x (+2), y (+4) | r, r | yes |
| 0x46 | x (+1), y (+3) | r, r | yes |
| 0x48 | x (+2), y (+4) | r, r | yes |
| 0x51 | mouse x (+1), mouse y (+3) | w, w | no |
| 0x53 | arg1 (+2), arg2 (+4) | r, r | yes |
| 0x73 | arg1 (+1), arg2 (+3) | r, r | yes |
| 0x87 | destination (+1) | w | no |
| 0xA0 | x (+1), y (+3) | r, r | yes |
| 0xA1 | x (+2), y (+4) | r, r | yes |

### 0x01 — conditional branch (handled before the switch)

size 11 bytes · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| arg1 | +2 | r | always a variable index | 0x40A2CF–0x40A2E7 |
| arg2 | +4 | r | `byte@+1 & 0x10` | 0x40A2E3–0x40A2EF |

The byte at `+1` is the branch type (`& 0x0F` selects 1 GE, 2 LE, 3 EQ, 4 NE, 5 GT, 6 LT); the
taken-branch target is the dword at `+6`.

### 0x02 — choice

size depends on the choice strings · 2 index operands **per choice**; `t` = first byte of the
trailer (the byte after the choice string)

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| availability u16 | t+1 | r | `byte@t+0 != 0` | 0x40FD3B |
| second value u16 | t+8 | r | `byte@t+7 != 0`, and only for a type-3 trailer (`byte@t+3 == 3`) | 0x40FDB8–0x40FDDC |

Type-3 trailer layout: `t+0` first condition, `t+1..2` first u16, `t+3` trailer type, `t+4` byte
field, `t+5..6` word field, `t+7` second condition, `t+8..9` second u16, `t+10` unread. All 128
corpus choices are type 3.

### 0x03 — variable heap op family

size 8 bytes · 2 index operands, selected by the type byte at `+1`

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| destination | +2 | `w` for types 1, 4, 6 · `rw` for types 2, 3, 5 | always a variable index | 0x40E6D5, 0x40E732, 0x40E747, 0x40E769, 0x40E77E, 0x40E7A8, 0x40E7C5, 0x40E7EF, 0x40E7FD, 0x40E832 |
| source | +5 | r | `byte@+4 != 0` — **except type 4, where the word is always an index** (the condition only selects one vs two dereferences) | 0x40E6BE, 0x40E726, 0x40E75D, 0x40E794, 0x40E7B9, 0x40E7D8, 0x40E810 |

Per-type detail (switch jump table `jpt_40E65B` @`0x4109E0`):

| type | case entry | operation | destination `+2` | source `+5` |
|---|---|---|---|---|
| 0 | 0x40E662 | `memset(heap, 0, 0x7D0)` — whole-heap reset, **no index operands** | — | — |
| 1 | 0x40E6B5 | `heap[dest] = src` | w @0x40E6D5 | r @0x40E6BE |
| 2 | 0x40E71D | `heap[dest] += src` | rw @0x40E732, 0x40E747 | r @0x40E726 |
| 3 | 0x40E754 | `heap[dest] -= src` | rw @0x40E769, 0x40E77E | r @0x40E75D |
| 4 | 0x40E78B | `heap[dest] = heap[src]` when `byte@+4 == 0`, else `heap[dest] = heap[heap[src]]` | w @0x40E7A8 (double deref), 0x40E7C5 (single) | r @0x40E794, 0x40E7B9; inner access @0x40E79C |
| 5 | 0x40E7CF | `if (src) heap[dest] %= src` | rw @0x40E7EF (read) + 0x40E7FD (write) | r @0x40E7D8 |
| 6 | 0x40E807 | `if (src) heap[dest] = rand() % src` | w @0x40E832 | r @0x40E810 |

Corpus reality: only types 0 (2×), 1 (16 763× flag 0 + 11× flag 1), 2 (37×) and 3 (4×) occur;
no type 4/5/6 anywhere in Rio.arc/Chip.arc.

### 0x0C — read timer completion

size 4 bytes · 1 index operand

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| destination | +1 | w | always a variable index | 0x40EB6E |

Writes `heap[+1] = (dword_4FAFF4 != 0)`.

### 0x0D — fill variable range

size 8 bytes · 1 index operand + 2 non-slot words

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| start of range | +1 | w | always a variable index; the opcode writes `count` consecutive slots from here | 0x40E8A6–0x40E8BD |
| range length | +3 | — | **not a slot** — a signed count | 0x40E8A0 |
| value written | +5 | — | **not a slot** — the value stored into each slot | 0x40E8B1 |

The only bound check is `start+i >= 0`: the compared register `esi` is zeroed every dispatch
iteration (`xor esi, esi` @0x40B4DB), so it is not compared against any variable.

### 0x23 / 0x27 — sprite-linked voice (one shared body)

size 11 + strlen · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| first position word | +2 | r | `byte@+1 & 0x01` | 0x40E145–0x40E151 |
| second position word | +4 | r | `byte@+1 & 0x02` | 0x40E162–0x40E16B |

`byte@+1` doubles as the sprite-slot number, so indirection and a named sprite slot are mutually
exclusive.

### 0x33 — read voice position

size 8 bytes · 3 index operands, all writes

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| minutes | +1 | w | always a variable index | 0x40E5A6 |
| seconds | +3 | w | always a variable index | 0x40E5C6 |
| milliseconds | +5 | w | always a variable index | 0x40E5DE, 0x40E618 |

### 0x43 — load .anm animation into a character slot

size 8 + strlen · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| x | +2 | r | `byte@+6 & 0x01` | 0x40AFA8, 0x40B12B, 0x40B2FF |
| y | +4 | r | `byte@+6 & 0x02` | 0x40AFD3, 0x40B14C, 0x40B32A |

The body resolves each position three times (one per load path), hence three addresses per operand.

### 0x46 — load background .wip

size 11 + strlen · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| x | +1 | r | `byte@+9 & 0x01` | 0x40B6F9 |
| y | +3 | r | `byte@+9 & 0x02` | 0x40B717 |

### 0x48 — load static sprite .wip/.msk

size 13 + strlen · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| x | +2 | r | `byte@+10 & 0x01` | 0x40C077 |
| y | +4 | r | `byte@+10 & 0x02` | 0x40C0A2 |

### 0x51 — read two words into variables

size 6 bytes · 2 index operands, both writes

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| mouse x | +1 | w | always a variable index | 0x40EC10 |
| mouse y | +3 | w | always a variable index | 0x40EC2C |

### 0x53 — message output with args

size 7 + strlen · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| arg1 | +2 | r | `byte@+1 & 0x01` | 0x40F40A |
| arg2 | +4 | r | `byte@+1 & 0x02` | 0x40F424 |

### 0x73 — load inlay .wip/.msk

size 11 + strlen · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| arg1 | +1 | r | `byte@+9 & 0x01` | 0x40BB4A |
| arg2 | +3 | r | `byte@+9 & 0x02` | 0x40BB65 |

### 0x87 — movie flag to variable

size 4 bytes · 1 index operand

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| destination | +1 | w | always a variable index | 0x40F36A |

### 0xA0 — background position

size 7 bytes · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| x | +1 | r | `byte@+5 & 0x01` | 0x40B873 |
| y | +3 | r | `byte@+5 & 0x02` | 0x40B88B |

### 0xA1 — character slot position

size 8 bytes · 2 index operands

| operand | at | access | condition | verified at |
|---|---|---|---|---|
| x | +2 | r | `byte@+6 & 0x01` | 0x40C41C |
| y | +4 | r | `byte@+6 & 0x02` | 0x40C441 |

### Corpus note for this section

Every `condition` above is false throughout Rio.arc/Chip.arc (`hv_gate_hist.py`), so in practice only
the "always a variable index" rows are exercised.

## Fixed slots accessed inside case bodies — verified (one row per opcode/slot pair)

`access` is the **engine's** access. Addresses are the instructions verified in the IDB.

| opcode | slot | access | what it is | verified at |
|---|---|---|---|---|
| 0x03 | 930 | r | change check (entry load + compare after the op) | 0x40E63B, 0x40E83C |
| 0x03 | 978 | r | `arg1 == 0x3D2` special case, and the value copied to `4FB060` | 0x40E627, 0x40E68C, 0x40E6A0, 0x40E6EC, 0x40E700, 0x40E857 |
| 0x03 | 991 | r | passed to `sub_45B6E0` | 0x40E630, 0x40E874 |
| 0x41, 0x42 | 979 | r | special-font select | 0x40A404, 0x40A869 |
| 0x41, 0x42 | 993 | r | timer duration in ms | 0x40A6F4, 0x40A70B, 0x40ABC7, 0x40ABDC |
| 0x04 | 992 | r | wait / re-run flag (also read by `sub_434D50`) | 0x40F964 |
| 0x05, 0x4C, 0x4D, 0x67, 0x70, 0x76, 0x82 | 996 | r | A/B switch variable | 0x40EABC, 0x40CB1E, 0x40CD9E, 0x40CF17, 0x40D4BA, 0x40BE64, 0x40F1B6 |
| 0x4A | 996 | rw | reads it, then clears it to 0 — the only engine write | 0x40CFF6 (r), 0x40D041 (w) |
| 0x4E (arg1 == 0x0A) | 941, 942 | r | x/y pair 1 snapshotted into 5FE118 | 0x40D842, 0x40D849 |
| 0x4E (arg1 == 0x0A) | 944, 945 | r | x/y pair 2 | 0x40D860, 0x40D86D |
| 0x4E (arg1 == 0x0A) | 947, 948 | r | x/y pair 3 | 0x40D884, 0x40D891 |
| 0x4E (arg1 == 0x0A) | 950, 951 | r | x/y pair 4 | 0x40D8A8, 0x40D8B5 |
| 0x83, 0x84 | 997 | r | controls `dword_4FEC3C` (script flag #997) | 0x40F28F, 0x40F2C4 |
| 0x86 | 931 | w | snapshot 1 | 0x40F0F3 |
| 0x86 | 932 | w | snapshot 2 | 0x40F148 |
| 0x86 | 933 | w | snapshot 3 | 0x40F14F |
| 0x86 | 998 | w | snapshot 4 | 0x40F10B |
| 0x88 | 956 | w | transition-flag snapshot | 0x40F39E |
| 0x88 | 957 | w | transition-flag snapshot | 0x40F3A5 |
| 0x8E | 995 | r | controls `92B110` (script flag #995) | 0x40F6A4 |
| 0xAB | 40 | w | pointer-position snapshot | 0x4101E2 |
| 0xAB | 41 | w | pointer-position snapshot | 0x4101E8 |
| 0xAD | 960..968 | r | pointer-anim struct + flag | 0x410252–0x4102AD |

Everything else in `sub_40A050` that references the array is either the base pointer
(`push offset heap_vars`, memcpy args) or an **address-taken range bound**
(`cmp reg, offset 6D98xx`), which is not a slot access:

* `sub_41BBE0`: `mov edi, offset slot942` @0x420919 + `cmp edi, offset slot955` @0x42095B and
  `mov edi, offset slot940` @0x42099E + `cmp edi, offset slot952` @0x420A5D → two block copies
  covering slots 940..953; plus real loads of 987 (×8), 988 (×3), 989 (×2).
* `Txt_DrawMainTextbox` 954/955, `sub_42CEB0` 972, `sub_4170D0` 975/980–986, `sub_4119A0` 987 w,
  UI readers of 994/995/996/999 — all fixed slots, listed in `hv_engine_slots.md` §2.

## Non-interpreter heap arithmetics — corrected

`Interp_VarArithOpcode` (0x409660, 8 indexed accesses) is **not** called from the interpreter:
its only callers are `sub_404130` @0x4070CF and `sub_407E60` @0x4094B3 (config/UI screens). The
script arithmetic paths live inline in the 0x03 body. (`hv_engine_slots.md` §3 grouped it as an
"arithmetic family" — that label is wrong.)

## Dispatch facts re-verified

* Implemented opcodes (own body): **127** (126 distinct entries; 0x23/0x27 share one) — plus the
  pre-switch 0x01 handler.
* Unimplemented opcodes (shared handler `0x4103BD`): exactly `0F-20 2A-2F 34-40 5A-5F 6A-6F 7A-80
  8F-9F AF-B0 C0-DF E1 EC-FE` — `binary_layout.md`'s DEAD list is **confirmed, no diff**.
* Loop head `loc_40B4DB` does `xor esi, esi` every iteration, so `esi` is 0 at every case entry.
  (This is why 0x0D's `cmp ecx, esi / jl` is a `< 0` check.)

## Corpus reality check

* 58 distinct slots < 1000 used; 288 distinct slots in 1001..1337 (`hv_script_slots.txt`,
  sections B and C — the file's header defines the columns and the tag legend).
* All indirection conditions are false → the conditioned operands are never exercised by
  Rio.arc/Chip.arc; only the `always a variable index` rows (0x01 arg1, 0x02 conditions, 0x03,
  0x0C, 0x0D, 0x33, 0x51, 0x87, and 0x03's 11 conditioned sources) appear.

## IDB state (applied + saved this session)

* `0x6D9148` is **not an item head** (it lives inside the `sjisText` blob @0x5FB5CE), so
  `set_cmt(0x6D9148, …)` is a silent no-op (ida.md gotcha #3) — the array's comment must go on the
  blob head.
* The blob-head comment (0x5FB5CE) was replaced with the corrected text (652 chars, 8 real
  newlines) that states the 3000-word space, the reset/save split, the correct system indices, and
  lists the INVALID old claims. `idb_save` done; backup at
  `/home/wscp/idb_backups/t13_heap_cmt_20260914_114120/`. Verified by grepping the saved `.id0`.
* Not restructured: `heap_vars` is still only a *name* on a non-head address. If a real array item
  with its own comment is wanted, use the T02 recipe (del_items over the 0x7D0 span + apply
  `unsigned __int16[1000]`) — note that item got absorbed back into the blob once already.
