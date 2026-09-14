# Binary-derived opcode layout table (authoritative)

Extracted **live from the interpreter** `sub_40A050` in CROSSCHANNEL.exe via IDA MCP — not from the
`llm_work/op_*.c` text dump (a lossy hand/LLM aid that contains at least one invented operand).

## How to read the table

```
OP | acc=<operand accesses> | lea=<address-of offsets> | ip=<instruction_ptr events>
```
* **All offsets are `ebp`-relative == `v10`-relative == (instruction offset − 1).**
  Instruction byte N of the opcode stream == `v10 + (N−1)`. `acc=4:w` means a 16-bit operand is read
  at instruction offset **+5..+6**.
* `b`/`w`/`d` = 1/2/4-byte read. `0:bw` = a byte and a word are both read at that offset (the byte is
  usually the low byte of the word, or a `cmp` against a derived value).
* `lea=` = `lea reg,[ebp+K]` — candidate address-of; NOT always the string start.
* `ok=` = instruction size/advance settled by another oracle (corpus alignment, or a callee);
  suppresses the `SIZE-UNKN` note for that row.
* `str=` = string start, **verified** (copy-idiom / `push reg` argument / `strstr` setup / raw corpus
  bytes). This is the column the checker trusts for string placement.
* `ip=`: `a<N>` = `add instruction_ptr, <N>` (decimal) → **fixed instruction size N**; `+1` = `inc`;
  `set` = ip assigned from a register (variable/absolute → string or jump opcode); `l<N>` =
  `lea reg,[lenreg+ebp+N]` (string-scan based advance); `-` = no ip write in the linear body.
* `*drift` = body reassigns `ebp` from the ip global, so later `[ebp+K]` disps are not `v10`-relative.

## Extractor caveats (why some rows are marked UNKNOWN)
1. Operands living in blocks reached only via a jump **outside the case's linear byte range** are
   missed. This is why the string start is absent for `41 53 42(2nd)` — verified separately by hand
   and by the corpus (see `findings.md`).
2. `ip=-` means "not found in the linear body", NOT "no advance". `07 0A 59 BA FF` are UNKNOWN and
   need a CFG walk that follows out-of-range targets.
3. `lea` is not always the string start (sometimes it is "address just past an operand"), e.g. `B2`
   reports `lea=1,2` where 2 is the string.
4. Bytes read at a given offset do not prove the *width* an encoder should emit; they prove the offset
   is live data. Width is taken from the widest read at the naturally-aligned base.

## Implemented vs unimplemented

127 implemented cases. The rest target one shared unimplemented handler:

```
DEAD: 0F-20; 2A-2F; 34-40; 5A-5F; 6A-6F; 7A-80; 8F-9F; AF-B0; C0-DF; E1; EC-FE
```
(`0x34` is therefore **dead** — the Rust arm for it is invented. Only `0x01` is handled *before*
the switch (test `cmp [eax], bl` @ `0x40A2C0`, where bl is always 1 due to `mov ebx,1` @
`0x40A247`): 11-byte entries, `add ip, 0Bh` at `0x40A347`. `0x00` falls into the main switch out of
range (index −2) → shared unknown-opcode handler — it is *not* implemented by the engine.)

## Table

```
02|acc=0:bw,1:w|lea=1|ip=a3setl1|DRIFT
03|acc=0:b,1:w,3:b,4:w|lea=-|ip=a8
04|acc=-|lea=-|ip=+1
05|acc=0:b|lea=-|ip=a3
06|acc=0:d|lea=-|ip=set
07|acc=0:b|lea=-|ip=-|ok=strlen+2 (advance inside sub_409BB0; corpus)|str=0
08|acc=-|lea=-|ip=a2
09|acc=0:b|lea=-|ip=set|str=0
0A|acc=-|lea=-|ip=-|ok=2 (corpus: 324/324 files, ret-like)
0B|acc=0:b|lea=-|ip=a3
0C|acc=0:w|lea=-|ip=a4
0D|acc=0:w,2:w,4:w|lea=-|ip=a8
0E|acc=0:b|lea=-|ip=a3
21|acc=0:b,1:w,3:b,4:w,6:d|lea=a|ip=set|str=A
22|acc=0:b,1:w|lea=-|ip=a5
23|acc=0:b,1:w,3:w,5:w,7:b,8:b|lea=9|ip=set|str=9
24|acc=-|lea=-|ip=a2
25|acc=0:b,1:b,2:b,3:b,4:w,6:b,7:w,9:b,a:b|lea=b|ip=set|str=B
26|acc=0:b|lea=-|ip=a3
27|acc=0:b,1:w,3:w,5:w,7:b,8:b|lea=9|ip=set|str=9
28|acc=0:b,1:b,2:w|lea=-|ip=a6
29|acc=0:b,1:w|lea=-|ip=a5
30|acc=0:b,1:w|lea=-|ip=a5
31|acc=0:b|lea=-|ip=a3
32|acc=0:b|lea=-|ip=a3
33|acc=0:w,2:w,4:w|lea=-|ip=a8
41|acc=0:w,2:b,3:b|lea=-|ip=set|str=4 (verified 2026-07-21: case @ 0x40A399, strlen loop after `add ebp,4`, size strlen+6)
42|acc=0:bw,2:b,3:b,4:b|lea=5|ip=l6set|str=5
43|acc=0:b,1:w,3:w,5:b|lea=6|ip=set|str=6
44|acc=0:b,1:b,2:b|lea=-|ip=a5
45|acc=0:b,1:b,2:b|lea=-|ip=a5
46|acc=0:w,2:w,4:d,8:b|lea=9|ip=set|str=9
47|acc=0:b|lea=-|ip=a3
48|acc=0:b,1:w,3:w,5:d,9:b,a:b|lea=b|ip=set|str=B
49|acc=0:b,1:b|lea=-|ip=a4
4A|acc=0:b,1:w,3:w|lea=-|ip=a7
4B|acc=0:b,1:w,3:w,5:d,9:w,b:d,f:d|lea=-|ip=a21
4C|acc=0:b,1:b,2:b,3:d|lea=-|ip=a9
4D|acc=0:b,1:b,2:w,4:w,6:w,8:w,a:w|lea=-|ip=a14
4E|acc=0:b,1:b,2:b|lea=-|ip=a5
4F|acc=0:b,1:b,2:b|lea=-|ip=a5
50|acc=0:b|lea=1|ip=l2set|str=0
51|acc=0:w,2:w|lea=-|ip=a6
52|acc=-|lea=-|ip=a3
53|acc=0:b,1:w,3:w|lea=-|ip=set|str=5
54|acc=0:b|lea=1|ip=l2set|str=0
55|acc=-|lea=-|ip=a2
56|acc=-|lea=-|ip=a2
57|acc=0:w,2:w,4:d|lea=-|ip=a10
58|acc=0:b,1:b,2:b,3:w,5:w|lea=-|ip=a9
59|acc=0:b|lea=1|ip=set|str=0
5A-5F|DEAD
60|acc=-|lea=-|ip=a2
61|acc=0:b|lea=1|ip=set|str=1
62|acc=-|lea=-|ip=a2
63|acc=0:b,1:b|lea=-|ip=a4
64|acc=0:b,1:w,3:w,5:w|lea=-|ip=a9
65|acc=0:w,2:w|lea=-|ip=a6
66|acc=0:b,1:w,3:w,5:b,6:w,8:d,c:w,e:d,12:d|lea=-|ip=a23
67|acc=0:b,1:b,2:b,3:d|lea=-|ip=a9
68|acc=0:w,2:w,4:w,6:w|lea=-|ip=a10
69|acc=0:b|lea=-|ip=a3
6A-6F|DEAD
70|acc=0:b,1:b,2:b,3:d|lea=-|ip=a9
71|acc=0:b|lea=1|ip=l2set|str=0
72|acc=-|lea=-|ip=a2
73|acc=0:w,2:w,4:d,8:b|lea=9|ip=set|str=9
74|acc=0:b|lea=-|ip=a3
75|acc=0:w,2:w,4:w,6:w|lea=-|ip=a10
76|acc=0:w,2:w,4:d,8:b,9:b,a:w,c:d|lea=-|ip=a18
77|acc=0:w,2:w,4:d|lea=-|ip=a10
78|acc=0:b,1:b,2:b,3:d|lea=-|ip=a9
79|acc=-|lea=-|ip=a2
7A-80|DEAD
81|acc=-|lea=-|ip=a3
82|acc=0:w|lea=-|ip=a4
83|acc=-|lea=-|ip=a2
84|acc=-|lea=-|ip=a2
85|acc=0:b|lea=-|ip=a3
86|acc=-|lea=-|ip=a3
87|acc=0:w|lea=-|ip=a4
88|acc=-|lea=-|ip=a4
89|acc=-|lea=-|ip=a2
8A|acc=-|lea=-|ip=a2
8B|acc=-|lea=-|ip=a2
8C|acc=0:w|lea=-|ip=a4
8D|acc=-|lea=-|ip=a2
8E|acc=-|lea=-|ip=a2
8F-9F|DEAD
A0|acc=0:w,2:w,4:b|lea=-|ip=a7
A1|acc=0:b,1:w,3:w,5:b|lea=-|ip=a8
A2|acc=0:b,1:w,3:w|lea=-|ip=a7
A3|acc=0:b,1:w,3:w|lea=-|ip=a7
A4|acc=0:b,1:w,3:w|lea=-|ip=a7
A5|acc=0:b|lea=-|ip=a3
A6|acc=-|lea=-|ip=a2
A7|acc=-|lea=-|ip=a2
A8|acc=0:b,1:b,2:b,3:b,4:b,5:w,7:w,9:w,b:w,d:w|lea=-|ip=a17
A9|acc=-|lea=-|ip=a2
AA|acc=0:b,1:b|lea=-|ip=a4
AB|acc=-|lea=-|ip=a2
AC|acc=-|lea=-|ip=a2
AD|acc=0:b,1:d,5:d|lea=-|ip=a11
AE|acc=-|lea=-|ip=a2
AF-B0|DEAD
B1|acc=0:w,2:w|lea=-|ip=a6
B2|acc=0:b|lea=1,2|ip=l4set|str=2
B3|acc=0:b|lea=-|ip=a3
B4|acc=0:b,1:b,2:w,4:w,6:d,a:b|lea=-|ip=a13
B5|acc=0:b,1:b,2:d|lea=-|ip=a8
B6|acc=0:w|lea=2|ip=set|str=2
B7|acc=0:b,1:w,3:w|lea=5|ip=set|str=5
B8|acc=0:b,1:b|lea=-|ip=a4
B9|acc=0:b,1:b|lea=-|ip=a4
BA|acc=0:w,2:w,4:b,5:b,6:b,7:b,8:b,9:w|lea=b|ip=-|ok=strlen+13 (verified 2026-07-21: case @ 0x40F9D9, `lea edx,[ecx+eax+0Dh]` @ 0x40FA80)|str=b
BB|acc=-|lea=-|ip=a2
BC|acc=0:b,1:b,2:b|lea=-|ip=a5
BD|acc=0:b|lea=-|ip=a3
BE|acc=0:b,1:b|lea=-|ip=a4
BF|acc=0:b,1:b,2:b,3:d|lea=-|ip=a9
C0-DF|DEAD
E0|acc=0:b|lea=1|ip=l2set|str=0
E2|acc=-|lea=-|ip=a2
E3|acc=-|lea=-|ip=a2
E4|acc=0:b|lea=-|ip=a3
E5|acc=-|lea=-|ip=a2
E6|acc=-|lea=-|ip=a3
E7|acc=0:w|lea=-|ip=a4
E8|acc=0:b|lea=1|ip=l2set|str=0
E9|acc=-|lea=-|ip=a2
EA|acc=0:b|lea=1|ip=set|str=1
EB|acc=-|lea=-|ip=a2
EC-FE|DEAD
FF|acc=-|lea=-|ip=-|ok=terminator
```

## Notable rows / verdicts

- **`0x25`** (corpus bug): live bytes `v10 0,1,2,3, w@4..5, 6, w@7..8, 9, A`, string at `v10+0xB`
  (confirmed by `lea eax,[ebp+0Bh]` at `0x40DA61` and again at `0x40DC4E`, feeding `_strstr`, with the
  ip write `mov dword_4F88F8, eax` at `0x40DC67`). → 11 fixed bytes, **string at instruction +12**.
  Rust `b,b,w,p,p,b,w,b,s` is wrong twice over (word at instr+5..6 modelled as two padding bytes,
  string one byte early). Correct: `b,b,b,b,w,b,w,b,b,b,s`.
  The `llm_work` dump's `*(_WORD *)(ctx->v10 + 2)` (instr+3) **does not exist in the binary**.
- **`0x07`**: only a string at `v10+0` (byte-copy idiom) — **no word operand**. Rust `w,s` truncates
  the script-name string in half (`"CGMODE"` → `Word 0x4743` + `"MODE"`). Correct: `s`.
- **The `b,b,b,d` family** (`4C 67 70 78 BF`, and `B5`): the engine consistently reads 3 live bytes
  then a dword at `v10+3` (instr+4..7). Rust is right only for `0x78`; `0x67`/`0x70` insert a bogus
  pad, `0x4C` shifts the dword one byte early, `0xB5` calls the dword padding, `0xBF` has both the
  wrong width and the wrong size.
- **`0xA8`**: five live bytes then **five** words (`v10 5,7,9,11,13`), then 1 pad. Rust
  `b,b,b,p,p,p,p,w,w,w,w,p` hides a whole word operand (instr+6..7) inside padding and reads the
  other four from the wrong offsets. Correct: `b,b,b,b,b,w,w,w,w,w,p` (17 ✓).
- **`0xB4`**: instr+1,+2 are read as bytes; Rust calls them `p,p`. Correct: `b,b,w,w,d,b,p` (13 ✓).
- **`0xA3`/`0xA4`**: instr+1 is a live byte; Rust says `p`. Correct: `b,w,w,p`.
- **`0x30`** / **`0x28`**: word at instr+2..3 / instr+3..4 modelled as `p,p`. Correct: `b,w,p,p` /
  `b,b,w,p`.
- **`0xB5`**: dword at instr+3..6 modelled as four padding bytes. Correct: `b,b,d,p,p` (8 ✓).
- **`0x29` size 5** (`a5`, not 6) and **`0xBF` size 9** (`a9`, not 7) — the two hard desync bugs.
- **`0x04`** size 1 ✓, **`0x06`** = dword target at instr+1, absolute ip set ✓.
- **`0x41`** (`w,b,b,s`, string at instr+5) has no `lea` in the linear body, but is confirmed by the
  corpus: 20 454 instances decode to correct Shift-JIS dialogue and all jump targets stay aligned.
- **`0xBA`** layout is fully consistent (`w,w,b,b,b,b,b,w` + string at instr+12); its advance lives
  outside the linear body.
- **UNKNOWN size** (no ip write in the linear body — needs a CFG walk): `07 0A 59 BA`.
  `0x0A` is suspicious in particular: Rust gives it `p` (size 2) with no evidence.

## Reproducing / extending

`validation/binary_check.py` parses the table above plus the `expand_opcode!` specs out of
`ccfkb_lib/src/opcodes/mod.rs` and reports SIZE / PAD-MISLABEL / WIDTH / STRING / ARMSET diffs.
Extraction script + gotchas: `validation/ida.md`. Corpus-side evidence: `validation/findings.md`.
