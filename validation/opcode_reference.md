# Opcode reference — what each opcode does

One entry per opcode implemented by the interpreter `sub_40A050` in CROSS†CHANNEL.exe, combining:

* **Layout** — from [`binary_layout.md`](binary_layout.md) (extracted live from the IDB; authoritative).
* **Current Rust arm** — `ccfkb_lib/src/opcodes/mod.rs` (`expand_opcode!`).
* **Behaviour** — [`../llm_work/behaviour.md`](../llm_work/behaviour.md) and [`../opcodes.md`](../opcodes.md)
  (outdated in places; where they disagree with the binary table, the binary wins).
* **Known bugs** — [`findings.md`](findings.md) / [`PLAN.md`](PLAN.md) §2.
* **Heap/script-variable operands** — every opcode that reads/writes `heap_vars` slots (verified
  operand offsets + gate bits, re-checked against the IDB) is tabulated in
  [`font_work/hv_opcode_access_map.md`](font_work/hv_opcode_access_map.md). Where this file's
  per-opcode prose omits or contradicts that table, the table wins.

## Legend

Field codes (instruction offsets counted from the opcode byte at +0):
`b` = 1-byte, `w` = 2-byte LE word, `d` = 4-byte LE dword, `s` = NUL-terminated Shift-JIS string,
`p` = preserved byte ("padding" — not read by the engine's linear body), `c` = choice list (0x02).
`str@+N` = verified string start.

Status: **✓** engine and Rust agree · **❌** known mismatch, fix in PLAN.md §2 · **⚠** partially unresolved.

Known-bug quick list (all ❌ below): `07 25 28 29 30 46 4C 67 70 A3 A4 A8 B3 B4 B5 BF`,
plus arm-set: `−0x34` (invented). (`0x00` is correctly absent from Rust — the engine itself does
not implement it; see below.)

---

## Control flow & variables (0x00–0x0F)

### 0x00 — not implemented (unknown-opcode error) ✓ (Rust's missing arm matches the engine)
There is **no** pre-switch path for 0x00. The only pre-dispatch test (`cmp [eax], bl` at
`0x40A2C0`) compares against `bl`, which is **always 1** — `mov ebx, 1` at `0x40A247` runs on every
loop iteration before the fetch — so it catches opcode **0x01 only**. Opcode 0x00 falls into the
main switch with index `0x00 − 2 = 0xFFFFFFFE > 0xFD` → `ja def_40A392` (`0x4103BD`) → shared
unknown-opcode handler: pushes script name (`byte_4F88E0`) + format string (`unk_4C8264`),
`jmp loc_40F927`, message box (debug builds) and termination via `sub_435360`. No payload, no ip
advance. Rust: no arm → `make_opcode` returns None → "Unknown opcode" + stop — **matches engine
semantics; do not add an arm** (retracts the earlier "+0x00 missing" item). The old "heap / mode
reset" attribution was wrong: that body (`memset(word_6D9148, 0, 0x7D0)`, clears 4FB008/4FB00C/
92B160, `sub_42FD20(10|92A474)`, `dword_4FB060 = word_6D98EC` at `loc_40E662`) is **0x03 type 0**.
Corpus: no script ever places 0x00 on an instruction boundary (no file starts with it; scripts end
`... 0A 00 FF <padding>`).

### 0x01 — conditional branch (11-byte entry) ✓ engine-confirmed
Layout `b,w,w,d,p`, size 11 (`add ip, 0Bh` at `0x40A347`; corpus: 1396 instructions, byte +10 == 0 in
all of them). Pre-switch handler, body at `loc_40A2CF`: entry test `cmp [eax], bl` @ `0x40A2C0`
(bl==1 always); arg1 = heap index w@+1..2; arg2 = w@+3..4 (heap-indirected when bit 5 of
branch_type is set, `test cl,10h` @ `0x40A2DE`); condition via 6-case switch on `branch_type &
0x0F`: 1 GE, 2 LE, 3 EQ, 4 NE, 5 GT, 6 LT (default: false). Advance: ip += 11 unconditional;
taken ⇒ ip += dword @+5..8 (`add eax,[ebp+5]` @ `0x40A353`). **Chains**: re-test at `0x40A35E`
— while the next opcode byte is also 0x01, further entries extend the branch table; make this
explicit in the model (PLAN §6).

### 0x02 — choice / conditional jump with strings ❌ (choice trailer)
Header `b,p` ✓ (n_choices @+1, then a separator byte). Each choice: `w` arg1 + `s` text + trailer.
Trailer's first 3 bytes are an **availability condition** (`flag ? heap[u16] != 0 : u16 != 0`),
*not* a jump target. Trailer length is **dynamic in arg5**: `3→8, 6→6, 7→strlen+2, else 1`.
Rust reads a blind 11-byte trailer (only correct for arg5 == 3 — all 128 corpus choices happen to use
3) and **panics** on truncated input. Fix: PLAN §3.2.

**Incomplete in prior text (verified 0x40FDB8–0x40FDDC):** a type-3 trailer carries a *second*
variable reference — byte `t+7 != 0` ⇒ `heap[u16 @ t+8]`, else the literal at `t+8` (stored to
choice-record +0x18). Trailer layout (t = byte after the choice string): `t+0` flag, `t+1..2`
availability u16, `t+3` trailer type (3), `t+4` byte, `t+5..6` word, `t+7` second gate,
`t+8..9` second u16, `t+10` unread.

### 0x03 — variable heap op family ✓
Layout `b,w,b,w,p`, size 8. type @+1: 0 = **the true heap/mode reset** (`memset(word_6D9148, 0,
0x7D0)`, clears movie flags 4FB008/4FB00C/92B160, `sub_42FD20(10|92A474)`,
`dword_4FB060 = word_6D98EC`; body at `loc_40E662`), 1 = `heap[arg1] = v`,
2 = `+= v`, 3 = `-= v`, 4 = double-indirect set, 5 = `if (v) %= v`, 6 = `if (v) = rand() % v`.
arg1 = heap index @+2..3; flag byte @+4; value u16 @+5..6 (heap-indirected when the flag bit is set).

### 0x04 — wait / re-run ✓
Size 1. Waits on `word_6D9908`: while clear it does **not** advance the IP (re-runs); once set it
does `dword_4FEC50 = 1` + `sub_422060(...)` and advances.

### 0x05 — movie overlay flag toggle ✓
Layout `b,p`, size 3. Toggles 4FAFF0/4FAFF4/4FAFEC around an active movie; redraw when not
mid-transition.

### 0x06 — absolute jump ✓
`d` target @+1..4, `ip = dword_4F88F4 + *(DWORD*)v10` (absolute within the current script). Rust
keeps one trailing pad byte (size 6); the engine takes an absolute target so fall-through size is not
observable.

### 0x07 — resource / script-name string op ❌
Engine: **string only**, `str@+1`, size strlen+2 (advance happens inside `sub_409BB0`). Copies the
string to `ctx->String2`. Rust `w,s` splits the first two characters of the name into a word
(`"CGMODE"` → `Word 0x4743` + `"MODE"`) — corrupts translatable text. Fix: `s` (PLAN §2).

### 0x08 — nop ✓
Size 2; second byte not read.

### 0x09 — call script (frame push) ✓
Layout `s`, str@+1, **engine-confirmed** (case at `0x40E99D`). Advances the IP *first*
(`ip = ip + strlen(str) + 2`, `lea edx,[ecx+eax+2]` @ `0x40E9B6`), then `sub_409DF0()` pushes a frame
— return address = `ip − base` (= the instruction **after** this entry), saved box state, and a
copy of the filename string — loads the callee `.fkb`, and the interpreter loop simply continues on
the new script image. So 0x09 is a true `call`, not a tail-call: it does **not** terminate anything.
Max depth 8; failure → error path (`loc_410526`).

### 0x0A — return (ret) ✓ — *engine-confirmed, was ⚠*
Size 2, no payload (the case body at `0x40EA41` reads **no** operands; the second byte is never
touched). `sub_409E80()`: if a call frame exists (`dword_4FEE58 != 0`) it pops it and sets
`ip = base + saved_return_address` (absolute restore — the case body itself advances nothing);
if the stack is empty it returns 0 and the case exits the interpreter: `sub_435360()` engine
shutdown, silently in non-debug builds, error dialog only when debug flag `dword_92B1FC` is set.
This is why every script ends with exactly one 0x0A — the top-level one has no frame and shuts
down the engine. Pairs with 0x09 as call/ret; Rust `p` (size 2) is correct, name it `ret`.

### 0x0B — start timer (seconds) ✓
Layout `b,p`, size 3. arg1 = seconds → deadline at 4FAFFC; completion flag 4FAFF4 is what 0x0C reads.

### 0x0C — read timer completion ✓
Layout `w,p`, size 4. `heap[arg1] = (dword_4FAFF4 != 0)`.

### 0x0D — fill variable range ✓
Layout `w,w,w,p`, size 8 (opcodes.md's "4-byte arg3" is wrong). Writes the u16 @+5..6 into `arg2`
consecutive heap slots starting at `arg1`. Not a debug print.
**Verified (0x40E894–0x40E8C6):** start = u16@+1, count = u16@+3, value = u16@+5; loop writes
`heap[start+i]`. The only bound check is `index < 0` (error dialog) — the compared register `esi`
is zeroed by the loop head (`xor esi, esi` @0x40B4DB); there is **no** check against slot 930.
Corpus use is exclusively the bulk-clear idiom: `[0..933) = 0` + `[934..1000) = 0`, sparing 933.

### 0x0E — stop movie + flag ✓
Layout `b,p`, size 3. `ctx->v6 = (arg1 == 0)`; clears movie flags and the 0x82 timer start;
`dword_4FEC80 = (arg1 != 0)`.

## Audio: voice pair & SFX (0x21–0x33)

### 0x21 — play .ogg on stereo voice pair ✓
Layout `b,w,b,w,d,s`, str@+11, size strlen+12. String = filename (".ogg" appended if missing).
Pan byte @+4 splits volume between the two channels; re-triggers if the same file is already loaded.

### 0x22 — voice-pair stop / volume ✓
Layout `b,w,p`, size 5. Stops both channels when `arg1 == 0 || arg2 == 0`, else sets volume.

### 0x23 / 0x27 — sprite-linked voice (shared arm) ✓
Layout `b,w,w,w,b,b,s`, str@+10, size strlen+11. String = .ogg name; arg1 = sprite index (-1 = none),
arg2 = frame; byte @+8: `101 ('e')` = seek/position mode, else volume/10. 0x23 uses state slot 0,
0x27 slot 1. Links the voice to a sprite frame; skipped during movies.
**Incomplete in prior text (verified 0x40E145–0x40E16B):** the two position words are gated
variable references — `byte@+1 & 1` ⇒ `heap[u16 @ +2]`, `byte@+1 & 2` ⇒ `heap[u16 @ +4]`; else the
words are literals. That byte is also the sprite index, so a script that uses indirection cannot
also name a sprite slot. All corpus instances have the byte = 0.

### 0x24 — audio mixer reset ✓
Size 2. `sub_434F50()` only (the helper 0x23/0x27 call before playback). opcodes.md's "sleep timers"
note is wrong.

### 0x25 — SFX play (13-slot table) ❌
Engine: `b,b,b,b,w,b,w,b,b,b,s`, **str@+12**, size strlen+12 (confirmed by `lea eax,[ebp+0Bh]` feeding
`_strstr`). Rust `b,b,w,p,p,b,w,b,s` mislabels the word @+5..6 as padding and starts the string one
byte early → parse desync + 40 bytes of round-trip loss in the corpus. Fix: PLAN §2.
Behaviour: arg1 = slot, arg2 = repeat/0xFF special, byte @+3 = persist flag, u16 @+5..6, byte @+7 =
start position (× 92A3C0 × 0.1), u16 @+8..9, volume/10 @+10, flag @+11. Gated by mute/movies.

### 0x26 — SFX stop ✓
Layout `b,p`, size 3. arg1 == 0xFF: stop + clear all 13 slots; else one slot.

### 0x28 — SFX seek ❌
Engine: `b,b,w,p`, size 6 (word @+3..4). Rust `b,b,p,p,p` mislabels the word as padding. Stores the
slot position and, when the slot (<10) is live, seeks via `sub_4638D0(0.1 * arg2 * 92A3C0)`.

### 0x29 — SFX stop with fade ❌ (size desync)
Engine: `b,w,p`, **size 5**. Rust says 6 → latent parser desync (0 occurrences in Rio.arc).
`sub_463C60(arg2)` when the slot is live; clears the slot flags either way.

### 0x30 — voice-pair pan ❌
Engine: `b,w,p,p`, size 5 (word @+2..3). Rust `b,p,p,p` mislabels it. Sets 4FEC90 = pan and applies
left/right volumes when stereo is active.

### 0x31 — SFX slot re-arm (persist) ✓
Layout `b,p`, size 3. Sets 6DB42C[slot] = 1 under live-table conditions.

### 0x32 — SFX slot re-arm (flag 6DB430) ✓
Same as 0x31 but sets 6DB430[slot].

### 0x33 — read voice position ✓
Layout `w,w,w,p`, size 8. All three words are heap variable indices: minutes, seconds (mod 60),
milliseconds of current voice playback; all -1 when idle.

## Sprites, backgrounds & transitions (0x41–0x4F)

### 0x41 — textbox, no speaker ✓ — *str@+5 now engine-confirmed*
Layout `w,b,b,s`, str@+5, size strlen+6. The extractor had missed it because the body does
`add ebp, 4` (`0x40A479`) and then runs the strcpy loop from the rebased `ebp` (case at
`0x40A399`; copy loop @ `0x40A488`, advance `ip = ip + strlen + 6` via `lea edx,[ecx+eax+6]`
@ `0x40A563`). arg1 = box layout id (w @+1..2), arg2 = box mode (b @+3, 3 = choice mode),
arg3 → 4FB070 (b @+4); string is the dialogue text. Resets the whole box state block.

### 0x42 — textbox, with speaker ✓
Layout `w,b,b,b,s,s`, first str@+6, size strlen(speaker)+strlen(text)+7. Speaker stored at
byte_4FE420; otherwise identical to 0x41.

### 0x43 — load .anm animation into character slot ✓
Layout `b,w,w,b,s`, str@+7, size strlen+8. String = .anm name; arg2/arg3 = position (heap flags in
arg4). Slot 0 = full-screen (also loads base .wip); other slots load their sprite's .msk mask.

### 0x44 — enable sprite frame ✓
Layout `b,b,b,p`, size 5. Marks frame arg2 of slot arg1: link flag = 1, active flag = arg3.

### 0x45 — flip sprite frame ✓
Same layout as 0x44. Sets the "just changed" marker + timestamp for frame arg2 of slot arg1.

### 0x46 — load background .wip ❌ (relabel only, engine-confirmed)
Engine: `w,w,d,b,s`, **str@+10**, size **strlen+11** (`lea eax,[edx+eax+0Bh]` @ `0x40B856`).
Fields: arg1 w@+1..2 (x), arg2 w@+3..4 (y) — rect extends to arg1+0x320 / arg2+0x258 (800×600);
dword @+5..8 is **live** (`cmp ecx,[ebp+4]` @ `0x40B776`, stored to `dword_5F8748`, passed to
`sub_4011D0`); flag b@+9 (bit 0 / bit 1 = heap-indirected arg1/arg2, `test [ebp+8],1/2`);
s@+10 = image basename — the engine **appends ".wip" at runtime** (constant
`dword_4C80DC = 0x7069772E` = ASCII ".wip", re-NUL via `byte_4C80E0 = 0`, written over the NUL of
the stack string copy @ `0x40B6EA`). Skip-if-unchanged is **generic**, not BLACK-specific:
`stricmp` vs the previously loaded name + dword compare (`0x40B75E`–`0x40B786`).
Rust `w,w,p,p,p,b,b,s` has the same string offset and total size (strlen+11) → **no desync,
pure relabel** (`p,p,p,b` @+5..8 → `d`). Corpus caveat: in Rio.arc, w@1-2, w@3-4, d@5-8 and b@9
are 0 in **1653/1653** instructions — only the string is live here, so re-encoding loses no bytes
in this corpus; the fix matters for other arcs. (Earlier "size strlen+9" and "largest data-loss
surface" claims were wrong / overstated.)

### 0x47 — background show/hide ✓
Layout `b,p`, size 3. arg1 == 2: recomposition pass; else `5F873C[0] = (arg1 != 0)`.

### 0x48 — load static sprite .wip/.msk ✓
Layout `b,w,w,d,b,b,s`, str@+12, size strlen+13. arg1 = slot, arg2/arg3 = position (heap flags in
arg5), dword = id, last byte selects whether the per-slot default 92B170[slot] is used. Re-load
skipped when name+id unchanged.

### 0x49 — static sprite active flag ⚠
Engine: two live bytes @+1,+2, size 4 (`ip=a4`). Rust models them as one word (`w,p`); behaviour.md:
"field layout is 1+1+1, not 2+1". Same total size (no data loss), width mislabel only.
`5F888C[slot] = (arg2 != 0)`.

### 0x4A — scene wipe / transition ✓
Layout `b,w,w,p`, size 7. Runs the wipe helpers, sets 600520/600530/600534 from the args, clears the
entire character-layer table.

### 0x4B — transition entry (move/anim) ✓
Layout `b,w,w,d,w,d,d,p`, size 21 (no null between arg4 and arg5). Fills a transition entry in the
84-stride table; arg1 > 100 switches to slot arg1-100 in alternate mode.

### 0x4C — scene transition (variant 1) ❌
Engine: `b,b,b,d,p`, size 9 (dword @+4..7). Rust `b,b,d,p,p` reads the dword one byte early → wrong
value, size coincidentally correct. Stores args into 4F50E8..4F50FC; `sub_439EF0(1)` in movie state;
clears the character-layer table.

### 0x4D — transition effect parameters ✓
Layout `b,b,w,w,w,w,w,p`, size 14. arg1 = effect type (5 = static noise), arg2 = secondary control,
arg3..arg7 → 4F5ABC..4F5AC8. Skipped in movies unless arg2 == 0xFF.

### 0x4E — wipe particle effect ✓
Layout `b,b,b,p`, size 5. arg1 = type (0 off, 1..8/11..16 wipes, 9/10 reset), arg2 = direction,
arg3 = force regen; fills the 11-stride particle table with rand()-based data.

### 0x4F — clear sprite frame markers ✓
Same layout as 0x44. Clears 61E51C/61E528 for frame arg2 of slot arg1; sets 61DB98[slot] = 1.

## Resources & message window (0x50–0x59)

### 0x50 — load .tbl ✓
Layout `s`, str@+1. String + ".tbl" → `sub_421940()`; error path on failure.

### 0x51 — read two words into variables ✓
Layout `w,w,p`, size 6. `heap[arg1] = word_4FFF16`, `heap[arg2] = word_4FFF18` (likely mouse coords).

### 0x52 — unload .tbl ⚠
Size 3 ✓ (`ip=a3`). Engine reads nothing in the linear body; Rust keeps a Byte @+1 (may be read by
the callee `sub_421B70`).

### 0x53 — message output with args ✓
Layout `b,w,w,s`, str@+6, size strlen+7. `sub_4254B0(arg2, arg3, string)`; args are heap indices when
arg1 bits are set; error dialog on failure.

### 0x54 — load .msk ✓
Layout `s`, str@+1. String + ".msk" → `sub_401310()` into 4F5ED8/4F5EDC.

### 0x55 — free .msk ✓
Size 2. Frees 4F5ED8, zeroes 4F5ED4..4F5EEC.

### 0x56 — message subsystem state ✓
Size 2. `sub_425600()` (same family as 0x53's callee).

### 0x57 — movement block setup ✓
Layout `w,w,d,p`, size 10. Stores arg1/arg2/arg3 into 804604/804608/80460C; 8045F8 = 1.

### 0x58 — per-slot value pair ✓
Layout `b,b,b,w,w,p`, size 9. `61DBC4[31708*arg1 + 6*arg2] = arg4`, `61DBC8[...] = arg5`.

### 0x59 — preload .wip ✓
Layout `s`, str@+1. String + ".wip" → `sub_41BA40()`; no trailing 4-byte arg (opcodes.md is wrong).

## Movies, inlays & textbox fades (0x60–0x79)

### 0x60 — release wipe resources ✓
Size 2. Frees 5FE0E0/5FE0E8, zeroes the fields.

### 0x61 — load/start movie ✓
Layout `b,s`, str@+2, size strlen+3. arg1 = mode (1/2 plain, 3 full-screen, 4 inlay). Path built from
PathName; failures terminate the script.

### 0x62 — cancel transition ✓
Size 2. 4F5AB0 = 0; copies 92AE9C/92AEA0 → 92A488/92A48C; 4FB030 = 1.

### 0x63 — static sprite flag ✓
Layout `b,b,p`, size 4. `5F8894[slot] = (arg2 != 0)`.

### 0x64 — sprite transform ✓
Layout `b,w,w,w,p`, size 9. arg2/100 = scaleX, arg3/100 = scaleY, arg4/10 = rotation; identity values
reset the slot instead of storing.

### 0x65 — transform origin + re-apply ✓
Layout `w,w,p`, size 6. 4E6D6C = arg1, 4E6D70 = arg2; re-applies stored transforms to slots 0..5.

### 0x66 — inlay entry (23 bytes, no trailing null) ✓
Layout `b,w,w,b,w,d,w,d,d`, size 23. Fills the 5F883x inlay entry for slot arg1; arg6 active (0
disables); centre from 4E6D74/4E6D78.

### 0x67 — scene transition (variant 2) ❌
Engine: `b,b,b,d,p`, size 9 (dword @+4..7). Rust `b,b,p,d,p` inserts a bogus pad before the dword →
the engine's live byte @+3 gets zeroed on re-encode (6 bytes lost in the corpus). Parallel to 0x4C
but into 5F9068..5F907C, `sub_439EF0(2)`.

### 0x68 — background zoom ✓
Layout `w,w,w,w,p`, size 10. arg1/100 & arg2/100 = scales (<1.0 → "bg" error dialog, forced to 1.0),
arg3/arg4 = centre.

### 0x69 — movie state byte ✓
Layout `b,p`, size 3. `dword_92B0BC = arg1`.

### 0x70 — scene transition (variant 3) ❌
Engine: `b,b,b,d,p`, size 9 — same fix as 0x67. Parallel to 0x4C/0x67: `sub_439EF0(3)`, branch on
5F8794[0]/5F87E4[0], clears the character-layer table.

### 0x71 — filename resource op ✓
Layout `s`, str@+1. String → `sub_42CAE0()`; error path on failure.

### 0x72 — filename resource op (counterpart) ⚠
Size 2 ✓ (`ip=a2`); no reads in the linear body (callee `sub_42CC40`). Rust `p` matches.

### 0x73 — load inlay .wip/.msk ✓
Layout `w,w,d,b,s`, str@+10, size strlen+9. String = base name; arg1/arg2 = position (heap flags in
arg4), dword = id (4FED9C). Re-load skipped when name+id unchanged.

### 0x74 — inlay stop ✓
Layout `b,p`, size 3. 4FED98 = (arg1 != 0); when movie inlay mode (92B0B4 == 4) is running: stop the
movie, free the mask, `sub_44A6E0()`.

### 0x75 — inlay move/resize ✓
Layout `w,w,w,w,p`, size 10. arg1/arg2 = new top-left, arg3/arg4 = size delta; out-of-bounds → error
dialog and inlay disabled.

### 0x76 — inlay fade setup ✓
Layout `w,w,d,b,b,w,d,p`, size 18. arg1/arg2 → 4FEE28/4FEE2C, dword = duration → 4FEE30;
4FEE1C = 1 when duration != 0.

### 0x77 — inlay move animation ✓
Layout `w,w,d,p`, size 10. Start = current inlay origin, offset = target − current, dword = duration.

### 0x78 — textbox fade start ✓
Layout `b,b,b,d,p`, size 9 — the only correct member of the `b,b,b,d` family. arg1 (0..100 percent,
must be non-zero) → 4F6EA4; arms 92B2D8/92B2DC/92B2E8.

### 0x79 — textbox fade cancel ✓
Size 2. 4F6E90 = 0; `sub_418980(1)`; resets 92B2DC/92B2E0/92B2E4.

## Timers, flags & misc (0x81–0x8E)

### 0x81 — no-op ✓
Size 3; only advances the IP.

### 0x82 — start timer ✓
Layout `w,p`, size 4. arg1 = duration → 92B148; skipped in movie state.

### 0x83 — resume ✓
Size 2. `sub_422A70(1, v405 == 1)`; pair with 0x84.

### 0x84 — pause ✓
Size 2. `sub_422A70(0, v405 == 1)`; clears 4FAFE0/5004E8.

### 0x85 — flag setter ✓
Layout `b,p`, size 3. `dword_4FEC30 = arg1` (gates 0xE3).

### 0x86 — state snapshot ✓
Size 3. word_6D988E = 92AE98, 6D9914 = (movie active), 6D9890 = `sub_463920()` result,
6D9892 = (4F88D0 != 0).

### 0x87 — movie flag to variable ✓
Layout `w,p`, size 4. `heap[arg1] = (4FB00C != 0 || 4FB008 != 0)`.

### 0x88 — transition-flag snapshot ✓
Size 4. word_6D98C0 = (4F50F0 != 0), word_6D98C2 = (5F9070 != 0).

### 0x89 — full state reset ✓
Size 2. Clears movie flags, `memset(&dword_6DB528, 0, 0x12900C)`, clears the 0x82 timer block.

### 0x8A — single call ✓
Size 2. `sub_432E10(1)`.

### 0x8B — single call ✓
Size 2. `sub_4258F0(1, 1)`.

### 0x8C — textbox state preset ✓
Layout `w,p`, size 4. `dword_4FB000 = arg1` (part of the box state block snapshotted by 0x41/0x42).

### 0x8D — box state op ✓
Size 2. 4FEC50 = 1; `sub_422060(±1, 3)` (sign from ctx->v405).

### 0x8E — flag setter ✓
Size 2. `dword_92B110 = 1` when `word_6D990E` is clear.

## Positional SFX, movie params, cursor & pointer (0xA0–0xAE)

### 0xA0 — background position ✓
Layout `w,w,b,p`, size 7. arg1/arg2 → 5F874C/5F8750 (+800/+600 rectangle); transform-dirty flag when
5F87E0[0] set.

### 0xA1 — character slot position ✓
Layout `b,w,w,b,p`, size 8. arg2/arg3 → 5F88AC/5F88B0 (heap flags in arg4); rectangle extended by the
sprite size from `sub_4529B0`.

### 0xA2 — positional SFX position ✓
Layout `b,w,w,p`, size 7. 92B328 entry (13-stride): active = 1, x = arg2, y = arg3.

### 0xA3 — positional SFX play ❌
Engine: `b,w,w,p`, size 7 (byte @+1 is live). Rust `p,w,w,p` mislabels it as padding.
`sub_465860({arg1, arg2})` (x, y).

### 0xA4 — positional SFX play (mode 2) ❌
Same fix as 0xA3. `sub_4658A0({arg1, arg2})`.

### 0xA5 — positional SFX stop ✓
Layout `b,p`, size 3. Zeroes the 92B328 entry for the slot; `sub_418980(1)`.

### 0xA6 — stop movie ✓
Size 2. Mode 3: `sub_459350()`, clears 5F873C[0]. Mode 4: also frees the inlay mask, clears 4FED98.
92B0B4 = 0 either way.

### 0xA7 — crosshair cursor ✓
Size 2. `sub_466720(&92B588, 16, aCross)`.

### 0xA8 — movie parameters ❌
Engine: `b,b,b,b,b,w,w,w,w,w,p`, size 17 — five live bytes then **five** words (@+6..7, +8..9, +10..11,
+12..13, +14..15). Rust `b,b,b,p,p,p,p,w,w,w,w,p` hides a whole word operand inside padding and reads
the other four from the wrong offsets. Only acts when 92B0F8 set: `sub_45AC40(arg1..arg3, arg4..arg7)`.

### 0xA9 — stop video ✓
Size 2. `sub_45AD00()` when 92B0F8 set.

### 0xAA — numbered cursor ✓
Layout `b,b,p`, size 4. arg1 (index; >=12 uses the second table base), arg2 (number) →
"<NAME>:<NN>" (upper-cased via LCMapStringA).

### 0xAB — pointer position snapshot ✓
Size 2. word_6D9198 = 4F6ED8, word_6D919A = 4F6ED4.

### 0xAC — cursor show/hide ✓
Size 2. `sub_466C70()` x2 (movie state) or `sub_466D60()` x2 (otherwise).

### 0xAD — pointer animation state ✓
Layout `b,d,d,p`, size 11. Copies six words (6D98C8..6D98D2) into a struct;
`sub_459D80(arg1, arg2, arg3, &struct, 6D98D8 != 0)`.

### 0xAE — single call ✓
Size 2. `sub_459F80()` (family of 0xAD).

## Effects & slot images (0xB1–0xBF)

### 0xB1 — background center ✓
Layout `w,w,p`, size 6. 4E6D7C = arg1, 4E6D80 = arg2.

### 0xB2 — load effect file ✓
Layout `b,p,s`, str@+3, size strlen+4. `sub_45C350(arg1, string)` when 92B0C4 set; error path on failure.

### 0xB3 — stop effect ❌
Engine: `b,p`, size 3 (byte @+1 is live). Rust `p,p` mislabels it → zeroed on re-encode (19 corpus
instructions). `sub_45C420()` when 92B0C4 set.

### 0xB4 — effect parameters ❌
Engine: `b,b,w,w,d,b,p`, size 13 (bytes @+1,+2 live). Rust `p,p,w,w,d,b,p` mislabels them (25 corpus
instructions). `sub_45C6A0(arg1, arg2, arg3, movie && arg4 == 2)` when 92B0C4 set.

### 0xB5 — effect frame step ❌
Engine: `b,b,d,p,p`, size 8 (dword @+3..6). Rust models the dword as four padding bytes.
Outside movies: `sub_45C6E0(arg2)`; inside movies: advances the effect object's frame directly.

### 0xB6 — append textbox text ✓
Layout `w,s`, str@+3, size strlen+4. Appends the string to the active Source buffer and re-validates
the box; arg1 selects the continuation mode; error dialog when no box is active.

### 0xB7 — load slot image (.wip + .msk) ✓
Layout `b,w,w,s`, str@+6, size strlen+7. String = base name (both extensions appended); arg1 = slot
(11-stride 6DB2x tables), arg2/arg3 = position; re-load skipped when the name is unchanged.

### 0xB8 — slot image show/hide ✓
Layout `b,b,p`, size 4. `6DB2F8[slot] = (arg2 != 0)`.

### 0xB9 — per-slot default ✓
Layout `b,b,p`, size 4. `dword_92B170[arg1] = arg2` (read by 0x48).

### 0xBA — colour/effect parameters ✓ — *ip advance now engine-confirmed*
Layout `w,w,b,b,b,b,b,w,s`, str@+12 (case at `0x40F9D9`). The advance was hidden in a branch target:
after the error-dialog path merges back, a strlen loop runs over the string and
`lea edx,[ecx+eax+0Dh]` @ `0x40FA80` sets `ip = ip + strlen(str) + 13` (stored at the shared
epilogue `loc_40B4D3`). Size = strlen+13, exactly matching Rust. arg1/arg2 into the 92B0C8 object;
`sub_45F3A0(arg3..arg7, 4FB050, arg8)`; error dialog with the string on failure.

### 0xBB — colour/effect reset ✓
Size 2. `sub_45F4D0()` (counterpart of 0xBA).

### 0xBC — advance animation frame ✓
Layout `b,b,b,p`, size 5. arg1 = slot (0 = background), arg2 = frame delta, arg3 = state; out-of-range
→ error dialog.

### 0xBD — flag setter ✓
Layout `b,p`, size 3. `dword_92B158 = (arg1 != 0)`.

### 0xBE — pair op ✓
Layout `b,b,p`, size 4. `sub_436BB0(arg1, arg2)`.

### 0xBF — textbox fade update ❌ (size desync)
Engine: `b,b,b,d,p`, **size 9** (dword @+4..7). Rust `b,b,b,w,p` has both the wrong width and the
wrong size (7) → latent parser desync. Updates 4F6EA4/4F6E98/4F6E94/4F6EA8 and 92B2E4 = 100*arg1 when
a fade (0x78) is active.

## Scene text & textbox sequence (0xE0–0xEB)

### 0xE0 — scene text ✓
Layout `s`, str@+1. String (max 78 chars, else error dialog) copied into the aCross buffer after
`sub_466C70()` + `sub_42AC00(0)`.

### 0xE2 — implicit resource op ✓
Size 2. `sub_419A90(sub_437140(v15), HIWORD(v15))` with v15 = opcode − 2 (0xE0). No payload.

### 0xE3 — implicit resource op (gated) ✓
Size 2. When 4FEC30 == 0 (see 0x85): `sub_418980(1)`, `sub_4348B0(1)`, `sub_452630(&4FEC28)`, then
`sub_419410(sub_437110(v15), HIWORD(v15))`.

### 0xE4 — textbox mode ✓
Layout `b,p`, size 3. 804614 = arg1, 804618 = (arg1 != 0); big memset of 4F5104 when arg1 != 0; peeks
at the next opcode byte (65/66/74 = 'A'/'B'/'J').

### 0xE5 — end textbox sequence ✓
Size 2. 4FB030 = 1; clears 804630/804624/804628.

### 0xE6 — no-op ✓
Size 3; only advances the IP.

### 0xE7 — mark table entry ✓
Layout `w,p`, size 4. `if (!byte_804638[arg1]) byte_804638[arg1] = 1;`

### 0xE8 — filename op ✓
Layout `s`, str@+1. `sub_418980(1)`, 4FB030 = 1; the string is skipped but not otherwise consumed.

### 0xE9 — no-string variant of 0xE8 ✓
Size 2. `sub_418980(1)`, 4FB030 = 1. (opcodes.md lists a `\0`-terminated string; the engine reads none.)

### 0xEA — .ogg file op ✓
Layout `b,s`, str@+2, size strlen+3. `sub_43DA50(arg1)`; the string is only the error-dialog label
(no .wip/.msk suffix appended here).

### 0xEB — .ogg file op (counterpart) ✓
Size 2. `sub_43DA00()`.

## Terminator

### 0xFF — end of script ✓
Size 1. Deliberate end marker: shared unknown-opcode path with its own message string, then
`sub_435360()` (engine shutdown) only when `dword_600518` is clear. Distinct from the default error.

## Dead ranges (no engine case — hit the shared unimplemented handler)

```
0F-20  2A-2F  34-40  5A-5F  6A-6F  7A-80  8F-9F  AF-B0  C0-DF  E1  EC-FE
```

* `0x34` is therefore **dead** — the Rust arm (`w,b,b,s`) is invented; a stray 0x34 is swallowed
  instead of reported. Fix: remove the arm, unknown opcode ⇒ hard error (PLAN §2/§3.4).
* Any byte in these ranges should be a positioned error from the Rust decoder, since the
  implementable set is now exactly known (127 switch cases + pre-switch 0x01; 0x00 falls to the
  unknown-opcode handler + 0xFF).

## Open items

~~`0xBA`'s ip advance and `0x41`'s string offset needed a CFG walk~~ — **both resolved by hand on
2026-07-21** (see the entries above; addresses recorded in [`ida.md`](ida.md)). Remaining:

* Reads inside helper callees are not modelled by the extractor, so "engine reads nothing" (0x52,
  0x72) is a lower bound: callees can only add operands, never remove them.
* Corpus is Rio.arc only; re-run the harness on the remaining arcs before trusting the "latent"
  classification of 0x29/0xBF and friends.
