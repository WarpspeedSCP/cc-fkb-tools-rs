# Font loading, glyph rasterizing & inline text formatting

Target: `CROSSCHANNEL.exe` (imagebase `0x400000`, IDA 9.2 + Hex-Rays). All addresses verified in the
live IDB; decompilations of every function named here are in this directory (`sub_*.c`), raw query
output in `q*.txt`. Marked **[inferred]** where behaviour was not directly observed.

Text is **not** pre-baked bitmaps: glyphs come from a GDI font through `GetGlyphOutlineA` and the
engine rasterizes the outlines itself into its own 8-bit buffers.

---

## 1. Config load / save (registry)

`sub_4620C0` (startup): `RegOpenKeyExA(hKey = 0x80000001, "software\\WillPlus\\CROSSCHANNEL", 0,
0xF003F /*KEY_ALL_ACCESS*/, …)` → **HKEY_CURRENT_USER**, and because the process is 32-bit it is
WOW64-redirected: on x64 Windows the real key is
`HKCU\Software\WOW6432Node\WillPlus\CROSSCHANNEL`.

| value | type / cbData | global | meaning |
|---|---|---|---|
| `FontName` | REG_SZ, 1024 | `g_szFontName` = `0x92A490` | primary UI/dialogue font face |
| `SpecialFontName` | REG_SZ, 1024 | `0x92A890` | second font (see §2) |
| `UseDefaultFont` | DWORD, 4 | `0x92A3E4` | **nonzero ⇒ use `FontName`** (name is misleading) |
| `FontEdge` | DWORD, 4 | `0x92A448` | edge/outline mode; also selects GDI quality (§3) |

`sub_462750` writes the same four names back from those globals — the game **overwrites them on
exit**, so edits made while it runs are lost. Read once at startup (no hot reload).
Other font-adjacent values read in the same block: `InstallType`, `AutoSkip`, `DisplayMode`,
`WndPos`, `MsgPos`, `MsgSpeed`, `EffectSkip`, `InstallDir`.

## 2. Which font a string uses — three tiers

Identical pattern at `0x414E7E`, `0x418A6E`, `0x4136B7` (and in the settings UI):

```c
if (textbox_state.field_50 /* 0x4FB050 */) name = g_szSpecialFontName;   // 0x92A890
else if (g_bUseDefaultFont /* 0x92A3E4 */) name = g_szFontName;          // 0x92A490
else                                       name = sub_45F0F0(1);         // built-in default
```

* `sub_45F0F0(i)` lazily loads a table of strings with `LoadStringA(hInst, *tbl, buf, 0x2000)` and
  returns `&unk_9296A0 + 0x2000*i`. ⇒ **the stock "MS Gothic" default lives in the EXE's RT_STRING
  resources**, not in code.
* `textbox_state+0x50` has exactly two writers, both inside the interpreter: `0x40A448` (opcode
  **0x41** body) and `0x40A8B3` (**0x42** body). Both execute

  ```asm
  xor  edx, edx
  cmp  ds:6D98EEh, cx      ; 6D98EE == heap_vars[979]   (heap_vars = 0x6D9148, word array)
  setnz dl
  mov  ds:4FB050h, edx
  ```

  `word_6D98EE` has **no code writers**, i.e. it is an ordinary scenario variable (index
  `(0x6D98EE-0x6D9148)/2 = 979`). ⇒ **the "special" font is switched on/off by script variable #979**
  — a native, per-textbox (`0x41`/`0x42`) second-font mechanism.

No availability check: the `EnumFontFamiliesExA` list (`sub_4653F0`, callback `Proc` @`0x465390`,
list head `0x92B21C`, cursor `0x92B220`) is only the settings-dialog dropdown; nothing validates a
registry name against it. Any installed family works.

## 3. Font creation — `sub_465120(name, &hFont, cellW, cellH, qualityFlag)`

Single `CreateFontIndirectA` wrapper in the module (only caller: the rasterizer + `sub_43D260`).
LOGFONT fields it sets:

| field | value | note |
|---|---|---|
| `lfHeight` | `cellH` | argument |
| `lfWidth` | `cellW` | argument (GDI mostly ignores this for TrueType) |
| `lfWeight` | 400 | FW_NORMAL, hardcoded |
| `lfItalic`, `lfUnderline`, `lfStrikeOut` | **0** | never set — no native italic request |
| `lfCharSet` | `0x80` | SHIFTJIS_CHARSET, hardcoded (byte 23) |
| `lfOutPrecision` / `lfClipPrecision` | 4 / 4 | OUT_TT_PRECIS / CLIP_DEFAULT_PRECIS |
| `lfQuality` | `4*(qualityFlag!=0)+2` | 2 = PROOF, 6 = CLEARTYPE_NATURAL (driven by `FontEdge`) |
| `lfPitchAndFamily` | 1 | |
| `lfFaceName` | `strcpy(name)` | **31 chars max** — longer names make `CreateFontIndirectA` fail, and the caller then draws nothing |

## 4. The one and only text rasterizer — `sub_415DD0`

```c
int __cdecl sub_415DD0(int surfaceId, int colorIdx, int x, int y,
                       int cellW, int cellH, const char *fontName,
                       const char *sjisText, int perCharAttrBase, int flag)
```

Callers: `sub_412E10`, `sub_413E80`, `sub_414320` (main textbox — reads `textbox_state` 0x4FB000),
`sub_418980`, `sub_423330`, `sub_425DF0`, `sub_4308F0`, `sub_431A20`, `sub_4332B0`, `sub_4355C0`,
`sub_4364A0`, `sub_436760`, `sub_437510`, `sub_438380`, `sub_45F530`, `sub_466890`.

Per character:

1. `_ismbclegal()` decodes 1 vs 2 bytes (`v42`/`v29` = bytes consumed).
2. glyph cache lookup `sub_45B100(cellW, cellH, code)` → hit returns cached bitmap + TEXTMETRICS via
   `sub_45B190(cellW, cellH, fontName, code, &tm)`; insert `sub_45B260`. **Cache key is
   (cellW, cellH, charcode)** — no font field in the key on lookup (font name only on insert).
3. miss → `GetTextMetricsA` + `GetGlyphOutlineA(hdc, code, 6 /*GGO_NATIVE|GGO_BEVEL*/, &gm, …)`
   twice (size, then data) with `MAT2` identity (`mat2 = {eM11=0x10000, eM12=0, eM22=0x10000}`),
   then the engine's own outline rasterizer `sub_416700` blits it (`sub_44FB70` / `sub_455470`).
4. **fallback**: `GetGlyphOutlineA == -1` (bitmap-only font) ⇒ clears `dword_4E6B2C`, recreates the
   font without the quality flag and renders with 4 × `TextOutA` at ±1 px + a final `TextOutA`.
   Outline (TrueType/OpenType CFF or TTF) fonts are what the engine expects.
5. `FontEdge` mode (`0x92A448 != 0`) adds the dilated/outline look: extra `TextOutA(x±1)`/`(y±1)` or
   a `sub_44F850` blit of a ±1-expanded rect — **disabled for the two smallest grids**
   (`cellW==6 && cellH==12`, `cellW==4 && cellH==8`).

Cell sizes actually used (from the ~30 call sites): main textbox & speaker plate **12×24**, small
lists (save/load etc.) **6×12**, config dialog **10×20**, tiny overlays **8×4 / 4×8 / 2×2**.

## 5. Half-width vs full-width — there is no width-class logic

Position is pure cell arithmetic on the **byte index**:

```c
v20   = cellW * byteIndex;                       // outline path
TextOutA(x0 + cellW * v10, y, …)                 // fallback path
x_ink = x0 + cellW * byteIndex + gm.ptGreyXOffset // glyph ink = own left side bearing
```

`v10` advances by *bytes* (`v10 += v29`), so a 2-byte SJIS char occupies exactly **2 cells** and any
single-byte char occupies **1 cell**. "Full-width" is an emergent property of DBCS byte count, not a
glyph table. Callers duplicate the same `cellW × strlen` math — e.g. name-plate centering at
`0x414E72` / `0x4136AB` / `0x418A62` (`lea esi,[ecx+ecx*2]; add esi,esi; add esi,esi; shr esi,1`
= ×12) and plate width `v104 = 12 * strlen(...)` in `sub_414320`. Space (0x20) and SJIS ideographic
space (0x8140 / 33088) are skipped for ink but still advance.

## 6. Native inline text formatting (the "easy formatting" channel)

Marker table `off_4E7FF8` (NULL-terminated `char*` array; strings at `0x4C7D20…0x4C7D8C`), matched
with `strncmp` **in table order** (so longest-first matters), consumed by `sub_42B070` (per-frame
print/pacing engine) and `sub_438CC0`. Verified index order:

| idx | marker | effect (from the switch in `sub_42B070`) |
|---|---|---|
| 0 | `\n` | expand/insert replacement text, sets `textbox_state.timer_done` |
| 1 | `%LC` | **line centre** → `textbox_state.field_84 = 1` (per-line alignment, see §6b) |
| 2 | `%C<hex4>` | **per-character colour**: 2 chars matched against a 16-entry palette `"0000","000F","5555","55FF","5F5F","77FF","7F7F","BFBF","F00F","F44F","F55F","F668","F8FF","FABF","FF5F","FFFF"` → per-char colour id |
| 3 / 4 | `%TS<n>` … `%TE` | **run style id** *n* on/off → per-char word array `0x4FDC20` (default `0xFFFF`); sets global `0x804638[n] = 1`; drives run-splitting in the drawer (§6b) |
| 5 / 6 | `%AS<n>` … `%AE` | A-effect on/off → per-char byte attr `0x4FC820[i] = n+1`. **Quirk:** the `%AE` reset handler is unreachable (case 29) because index 6 shadows index 29 in the match loop, so `%AS<n>` sticks until another `%AS<m>` |
| 7 / 8 | `%WS<n>` … `%WE` | W-effect on/off → per-char dword `0x4FB820[i] = n` (only if not already set by `%T`) |
| 11 | `%K` | marks text as read (`textbox_state.restore_id = 1`) — the back-log/skip bookkeeping marker |
| 12 / 13 | `%P` / `%p` | consumed, no state (line terminator; corpus: `%K%P` ends ~every dialogue string, 249 564 occurrences) |
| 14 | `%N` | newline-ish expansion (shares code with `\n`) |
| 15 / 19 | `%O` / `%E` | auto-advance/wait gating (`field_48 = field_3c = 1`, gated by `EffectSkip`) |
| 16 / 17 | `%FE` / `%FS` | F-effect end/start |
| 18 | `%LF` | line feed → `field_84 = 0` |
| 20 | `%V` | voice/movie related; writes `heap_vars[993]` |
| 21 / 22 | `%WS` / `%WE` | duplicate entries (harmless, shadowed by 7/8) |
| 23 | `%W` | wait at this character (records position in `0x4FB818`) |
| 24 | `%T<n>` | **per-character timing** → `dword_4FB820[charIdx] = n` |
| 25 / 26 / 27 | `%XS<n>` / `%XE` / `%X<n>` | **per-run text size**: `%XS<n>` sets `textbox_state.field_5c = n * 100/24` (n is a pixel size against the 24 px base; `n == 0 \|\| n == 99` ⇒ 24 ⇒ 100 %), `%XE` resets to 100 %, `%X<n>` parses and discards its param |
| 28 | `%as<n>` | A-effect variant (same handler as `%AS`) |
| 30 | `%FF` | form feed / page clear |
| 31 | `%<dd>` | bare `%` + 2 digits ⇒ sets `field_5c` to an **explicit scale percent** (default 100) |

Matching is a prefix walk (`strncmp(text, table[i], strlen(table[i]))`) in table order, so the first
match wins — that is why duplicates exist and why `%AE`/`%WS`/`%WE` at indices 21/22/29 are dead.

The pass strips markers from the visible text and fills **parallel per-character arrays**:
`unk_4FB418+i` (byte attr — colour id), `0x4FDC20+2i` (word param), `dword_4FB820+4i` (timing).
Those are what reach the rasterizer as `perCharAttrBase`:
`*(u8*)(idx + a10)` indexes table `unk_4E2C18` (4 bytes/entry). ⇒ **colour and timing are per-glyph;
font is per-call** (one font name per `sub_415DD0` call, i.e. per drawn string/textbox).

**Ruby markup**: `{漢字:よみがな}` — parsed by `sub_42AC00`, the only `{}` consumer (`{`=0x7B,
`:`=0x3A, `}`=0x7D). It removes the reading from the visible text and records up to 20 markers
(44-byte stride at `unk_4FB0B4`, count at `unk_4FB0A4`) with base position + reading; readings longer
than 0x1E bytes abort the parse. Corpus: 810 ruby instances in the Rio.arc scripts.

---

## 6b. Per-glyph attribute channels (session 2) — colour, scale, offsets, alignment

`sub_42B070` stamps one record per **byte** of visible text (`default:` case and the two DBCS paths):

| array | width | source marker | meaning |
|---|---|---|---|
| `0x4FB418[i]` | byte | `%C<hex4>` | glyph **colour index** (palette at `0x4E2C18`, RGB triples; the parallel palette at `0x4E2BD8` tints the plate via `sub_4658E0`) |
| `0x4FC820[i]` | byte | `%AS<n>` / `%as<n>` | A-effect attr |
| `0x4FDC20[i]` | word | `%TS<n>` … `%TE` | **run style id**, `0xFFFF` = unstyled |
| `0x4FB820[i]` | dword | `%T<n>`, else `%WS<n>` | per-char param (written only if still zero) |
| `0x4FCC20[i]` | dword | from `textbox_state.field_5c` (`%XS`/`%XE`/`%<dd>`) | **packed geometry**: bits 2–10 = x offset, bits 11–19 = y offset, bits 20–22 = line index, bits 23–31 = scale percent |
| `unk_4FB084[line]` | dword | `%LL` / `%LC` / `%LR` (`%LF` resets) | **per-line alignment**: 0 = left, 1 = centre, 2 = right |

Unpacking is visible in `sub_43D0F0` (line-measure helper):

```c
v10 = (geom >> 20) & 7;                 // line index
if (geom >> 23) *pXoff = (geom >> 2) & 0x1FF;
advance += (int)((double)(geom >> 23) * 0.01 * 12.0);
```

**The main dialogue drawer is `sub_436760`** (called from `sub_414320` with the per-char arrays for
the current line). Its behaviour:

1. split the line into runs of equal style word (`0x4FDC20`);
2. inside an unstyled run, group consecutive chars that share a scale, then per group call
   `sub_415DD0(3, colourIdx, x, y0 + ((geom>>11)&0x1FF), cellW, cellH, font, text, dst,
   &colourAttr[base+i])` with **`cellW = scale% × 12.0`, `cellH = scale% × 24.0`**;
3. advance `x += cellW * chars` and `UnionRect(outRect, …)` — the accumulated rect is returned and the
   caller blits exactly that region, so **layout adapts to whatever widths you produce**;
4. styled runs go through `sub_4364A0` instead: colour index `8*(slotState==2)+7` (i.e. 7 or 15) and a
   second pass over the same text with the buffer filled with `'_'` — the redacted/placeholder effect.

So natively the engine supports **per-glyph colour, per-glyph uniform scale, per-glyph y-offset,
per-line alignment** — but *not* per-glyph font (font is one value per `sub_415DD0` call) and not
per-glyph x advance independent of height.

Colour tables: `dword_4F33D8[16]` = runtime text palette used by the `colourIdx` argument (BGR:
0–4 white, 5 pink, 6 light blue, 7 orange, 8 peach, 9 violet, 10 near-black, 11 yellow, 12 salmon,
13 green-yellow, 14 orange-red, **15 = white**, the default for movie/effect text). `0x4E2C18[16]` /
`0x4E2BD8[16]` (RGB triples) are the per-glyph / plate palettes. All three live in writable `.data`,
so a translation can recolour by patching data only.

## 6c. Base cell metrics are two shared doubles — the highest-leverage width knob

Every scale-aware text routine computes its cell from two constants:

```
dbl_4D78D8 = 12.0   // base advance (cellW)  — 19 xrefs
dbl_4D78E0 = 24.0   // base height (cellH)  — 15 xrefs
users: sub_412E10, sub_414320, sub_418980, sub_4308F0, sub_431A20,
       sub_436760 (dialogue body), sub_43CB20 (wrap/repack), sub_43D0F0, sub_43D1C0
```

Changing `dbl_4D78D8` alone narrows advance while leaving height at 24 — glyphs are drawn from their
outlines at natural size and positioned by the cell, so this yields normal-height, narrow-advance text
(= the half-width look) with an **8-byte data patch**. Caveat: several call sites hardcode `12` as an
integer instead of using the constant (e.g. speaker-name `sub_415DD0(3,…,12,24,…)` in `sub_414320`,
`sub_418980`, `sub_437510`, plus the `×12` centering math at `0x414E72`/`0x4136AB`/`0x418A62`), so
those must be patched to the same value or centered strings/plates drift. A narrower blast radius:
redirect only the operand of `fld ds:dbl_4D78D8` at `0x436871` (dialogue body) to a private double in
a `.rdata` cave (zero runs ≥16 B exist, e.g. `0x4BA45C`, `0x4BD97C`).

## 7. Why changing the registry font appeared to do nothing

Ranked by likelihood:

1. **`UseDefaultFont == 0` ⇒ `FontName` is ignored entirely** (gate at `0x414E8D`, `0x418A7D`,
   `0x4136C6`); the engine then uses the RT_STRING default. The value must be **nonzero**.
2. **Wrong hive/view.** It is `HKCU` (not HKLM), read from a 32-bit process ⇒ on x64 Windows you must
   edit `HKCU\Software\WOW6432Node\WillPlus\CROSSCHANNEL`. Editing the non-redirected path with 64-bit
   `regedit`, or editing HKLM, has no effect.
3. **Clobbered on exit** — `sub_462750` writes these values back from memory; edit while the game is
   closed.
4. Face name > 31 chars ⇒ `CreateFontIndirectA` fails ⇒ `sub_415DD0` returns 0 and *no text is drawn*
   (looks like "nothing changed" if you also had a fallback path).
5. Startup-only read: needs a full restart.

## 8. What to patch / configure for an italic + half-width English patch

**Font swap (config only)** — `UseDefaultFont`=1, `FontName`/`SpecialFontName` = family names ≤31
chars, REG_SZ, in the WOW6432Node path, game closed. Use TrueType/OpenType outline fonts (bitmap /
ODBM fonts take the degraded 4×`TextOutA` path). Note `lfCharSet=SHIFTJIS`: GDI may substitute a face
with SJIS coverage; either pick a font with Japanese coverage or patch LOGFONT byte 23 at
`sub_415DD0`→`sub_465120` (`HIBYTE(lf[5]) = 0x80`, `0x465157`) to `0x01` (ANSI) for Latin-only text.

**Italic, cheapest first:**

1. **No patch — use the built-in second font.** Set `SpecialFontName` to your italic face and toggle
   `heap_vars[979]` from the scenario around the runs you want italic. This is the engine's own
   emphasis mechanism (`0x41`/`0x42` pick it up on the next textbox). Granularity = per textbox.
2. **Global synthetic italic, ~1 instruction.** In `sub_465120`, LOGFONT byte 20 (`lfItalic`) shares
   the dword written by `HIBYTE(lf[5]) = 0x80` at `0x465157`. Replace that byte-store with a dword
   store of `0x80000001` (bytes 20..23 = `01 00 00 80`) — same length, sets `lfItalic=1` and keeps
   `lfCharSet=0x80`. GDI selects the family's real italic face if present, otherwise shears the
   outline (TrueType synthetic oblique). **[inferred]** the synthetic-shear behaviour should be
   verified on the target font.
3. **Global shear at rasterize time.** The `MAT2` initialised just before the first
   `GetGlyphOutlineA` (`mat2=0x10000`, `mat2_4=0`, `mat2_12=0x10000`) is passed to both outline calls;
   setting `eM21` (the dword after `mat2_4`) to e.g. `0x199A` (0.1 in 16.16) slants every glyph.
   Only affects the outline path; ink may overhang the cell by ~1 px (edge mode already pads ±1).
4. Per-run italic inside one font is **not** achievable through SJIS code points (GDI maps DBCS via
   CP932 to Unicode; PUA is unreachable) — use (1).

**Half-width:**

* A Latin translation needs nothing disabled: ASCII = 1 byte = 1 cell, already half-width. The real
  issue is the textbox grid being **12 px advance at 24 px height** (0.5 em) — looks loose.
* **Best single knob (session 2): `dbl_4D78D8` (12.0)** at `0x4D78D8` — see §6c. One 8-byte data patch
  narrows advance for every scale-aware drawer while height stays 24; then fix the handful of integer
  `12` hardcodes that must agree (`push 0Ch` args + `×12` centering math).
* Narrower blast radius: redirect only `fld ds:dbl_4D78D8` at `0x436871` (dialogue body in
  `sub_436760`) to a cave double, leaving menus/plates untouched.
* Widest blast radius: shrink the `push 0Ch` immediates before `call sub_415DD0` (`0x414EBC`,
  `0x418AAC`, `0x413107`, …) **and** the matching `×12` centering math at `0x414E72` / `0x4136AB` /
  `0x418A62`, else centered text (speaker plate) drifts. Or scale `cellW` once on entry to
  `sub_415DD0` and accept mis-centering (left-aligned dialogue is unaffected).
* Force 2-byte SJIS glyphs into **one** cell: keep the byte advance (`v10 += v29`) but add a separate
  *glyph* counter used by the two position computations — `v20 = cellW * String` (outline path) and
  `a3 + cellW * v10` (fallback path). Then fix callers' `cellW × strlen` widths (≥3 centering sites +
  plate sizing) or boxes/centring are off by the DBCS char count.
* True proportional spacing: accumulate per-glyph advance from GLYPHMETRICS (`gm[2] + black width`,
  or `tm.tmAveCharWidth`) instead of `cellW × index` at those same two sites — same patch points, but
  every caller-side width assumption then needs to come from a measured width instead.

**Preserve while translating:** `%K%P` line suffixes (read-skip state), `%C<hex4>` colour codes,
`\n`/`%N`/`%LF` breaks, `%W` waits, `%T<n>` timings, `%FF` page clears, and well-formed
`{kanji:kana}` ruby (malformed / >30-byte readings abort the marker parse).

**Per-run italic hook (better than §8 item 1 for mixed content):** because `sub_436760` / `sub_4364A0`
issue one rasterizer call *per style run*, the font argument they pass (`v23`/`v17`, chosen by the
three-tier rule) can be made style-dependent with a small patch — e.g. in `sub_436760` at `0x43689x`
(`if (a9) v23 = &unk_92A890; …`) select `g_szSpecialFontName` whenever the run's style word
(`*(u16*)(a2 + 2*i)`) is not `0xFFFF`. Then `%TS<n>…%TE` becomes a native italic/emphasis markup with
correct run splitting for free, per-run rather than per-textbox.

## 9. Loose ends for the next session

* Where ruby readings are drawn (which cell size / font) — candidate: the small-grid callers of
  `sub_415DD0`; the marker pass records them at `unk_4FB0B4` (44-byte stride, count `unk_4FB0A4`) and
  `sub_414320` loops `v115 < unk_4FB0A4` before the body draw.
* `%AS<n>` (`0x4FC820`) and `%WS<n>`/`%T<n>` (`0x4FB820`) consumers at draw time are only partially
  mapped: `sub_414120`, `sub_4308F0`, `sub_414320` read `0x4FB820`; the animation/repack pass is
  `sub_43CB20` (reads and rewrites `0x4FCC20`, heaviest user of the two metric doubles).
* `0x804638[n]` byte states: `1` = set by `%TS<n>`, `2` = set by `sub_437EC0`/`sub_4382B0`; read as
  `==1` in `sub_437510` (movie text overlay, 12×24 at x=140) and `==2` in `sub_4364A0` (colour pick).
  Full consumer list in `q18.txt`.
* `_ismbclegal` has **37 call sites** (list in `q2.txt`) — the full set of byte-vs-glyph assumptions
  to audit before any UTF-8 conversion; the interpreter (`sub_40A050`) itself uses it via
  `sub_42AC00`/`sub_42BFA0`.
* Verify empirically: (a) `%XS<nn>` resize, (b) `%LC`/`%LR` per-line alignment, (c) that `%AE` really
  fails to clear `%AS`, from a scripted test case rather than only from the decompiler.

## 10. Symbols written back into the IDB (session 2)

Saved to `CROSSCHANNEL.exe.i64`. Old name → new name (functions keep their address, so this table is
the mapping for everything referenced above).

| address | old | new | confidence |
|---|---|---|---|
| `0x415DD0` | sub_415DD0 | `Txt_RenderText` | high (proto also applied) |
| `0x42B070` | sub_42B070 | `Txt_ParseMarkers` | high |
| `0x436760` | sub_436760 | `Txt_DrawDialogueLine` | high (proto applied) |
| `0x4364A0` | sub_4364A0 | `Txt_DrawStyledRuns` | high (proto applied) |
| `0x43D0F0` | sub_43D0F0 | `Txt_MeasureLineUpTo` | high |
| `0x414320` | sub_414320 | `Txt_DrawMainTextbox` | high |
| `0x4620C0` | sub_4620C0 | `Cfg_LoadFromRegistry` | certain |
| `0x462750` | sub_462750 | `Cfg_SaveToRegistry` | certain |
| `0x465120` | sub_465120 | `Font_CreateIndirect` | certain |
| `0x45F0F0` | sub_45F0F0 | `Res_GetResourceString` | high |
| `0x42AC00` | sub_42AC00 | `Ruby_ParseMarkers` | high |
| `0x45B100` | sub_45B100 | `GlyphCache_HasGlyph` | high |
| `0x45B190` | sub_45B190 | `GlyphCache_Lookup` | high |
| `0x45B260` | sub_45B260 | `GlyphCache_Store` | high |
| `0x416700` | sub_416700 | `Glyph_RasterizeOutline` | medium-high |
| `0x43CB20` | sub_43CB20 | `Txt_UpdateGlyphGeom` | medium |
| `0x437510` | sub_437510 | `Txt_DrawMovieText` | medium |
| `0x438CC0` | sub_438CC0 | `Txt_MarkerScan` | medium |
| `0x42BD50` | sub_42BD50 | `Txt_AppendGlyphRecords` | medium |

Data symbols (types applied where shown):

| address | new name | type / contents |
|---|---|---|
| `0x92A490` | `g_szFontName` | `char[1024]` registry FontName |
| `0x92A890` | `g_szSpecialFontName` | `char[1024]` registry SpecialFontName |
| `0x92A3E4` | `g_bUseDefaultFont` | `int` — nonzero ⇒ use `g_szFontName` |
| `0x92A448` | `g_bFontEdgeMode` | `int` outline/edge text mode |
| `0x4E6B2C` | `g_bOutlinePathOK` | cleared when `GetGlyphOutlineA` fails → `TextOutA` fallback |
| `0x6D98EE` | `g_varSpecialFontSel` | script variable **#979** in `g_scriptVars` — selects the special font (see §11) |
| `0x6D9148` | `heap_vars` | `unsigned short[1000]` script-variable array (§11) |
| `0x6D990A` | `g_varMarkerV` | script variable **#993** — written by the `%V` marker (see §11) |
| `0x4FB418` | `g_glyphColorIdx` | `u8[1024]` per-glyph colour index (`%C`) |
| `0x4FC820` | `g_glyphAttrA` | `u8[1024]` per-glyph A-attr (`%AS`) |
| `0x4FDC20` | `g_glyphStyleId` | `u16[1024]` run style id (`%TS`, `0xFFFF` = plain) |
| `0x4FB820` | `g_glyphParamT` | `int[1024]` per-glyph param (`%T` / `%WS`) |
| `0x4FCC20` | `g_glyphGeom` | `u32[1024]` packed geometry (x 2–10, y 11–19, line 20–22, scale% 23–31) |
| `0x4FB084` | `g_lineAlign` | `int[32]` per-line alignment 0 left / 1 centre / 2 right (`%LL %LC %LR`) |
| `0x4FB0A4` | `g_rubyCount` | ruby marker count (≤20) |
| `0x4FB0B4` | `g_rubyMarkers` | ruby markers, 44-byte stride |
| `0x4F33D8` | `g_textPalette16` | `u32[16]` runtime text palette (COLORREF), 15 = white |
| `0x4E2C18` | `g_glyphColorRGB16` | `u32[16]` per-glyph colour table (RGB order) |
| `0x4E2BD8` | `g_plateColorRGB16` | `u32[16]` plate/tint colour table |
| `0x4D78D8` | `g_baseCellW_12_0` | `double` **12.0** — base text advance, 19 xrefs |
| `0x4D78E0` | `g_baseCellH_24_0` | `double` **24.0** — base text height, 15 xrefs |
| `0x4E7FF8` | `g_txtMarkerTable` | `char*[32]` inline `%` marker vocabulary (§6) |
| `0x804638` | `g_styleSlotState` | `u8[1024]` style-slot state (1 = `%TS`, 2 = set by `sub_437EC0`/`sub_4382B0`) |
| `0x92B0CC` | `g_glyphCacheList` | glyph cache block list head |
| `0x92B21C` | `g_fontEnumList` | `EnumFontFamiliesExA` result (settings dropdown only) |

Inline comments added at `0x465157` (lfItalic patch point), `0x436871` (`fld g_baseCellW_12_0`) and
`0x414E8D` (the `UseDefaultFont` gate).

## 11. The script-variable heap (`heap_vars`) and its hard-coded slots (session 3)

> **INVALID AS OF THE RE-VERIFICATION (2026-XX).** Full evidence: `font_work/hv_opcode_access_map.md`
> + `font_work/hv_engine_slots.md`. Corrections to this section:
> * **size** — the variable space is **3000 words** (`0x6D9148..0x6DA8B8`). The "1000 words" below
>   only covers the block that the reset clears. `sub_419290`/`sub_419350` (save000) save/load
>   `&unk_6D9918, 2, 2000` = slots 1000..2999, and the corpus uses 1001..1337.
> * **"Bounds proven … `cmp ecx,3E8h`/`imul edx,3E8h`"** is wrong: `sub_41BBE0`'s `3E8h` is a
>   `%1000`/`-1000` computation. Also `memset(&sjisText[908154], 0, 2000)` is in `sub_409660`, a
>   config/UI helper (`sub_404130`/`sub_407E60`), not the interpreter.
> * **"56 slots addressed by absolute address"** — only **53** are real load/store accesses; slots
>   940 and 952 (and the `sub_41BBE0` rows of 954/955) are `cmp reg, offset …` address comparisons
>   used as block-copy loop bounds.
> * **row `930`** is read by the **0x03** body (entry change-check), not by `sub_41BBE0`; the table
>   in §11 is missing 930 and 991 for `sub_40A050`.
> * the rest of the slot table (954/955, 972, 975, 977–989, 991–999) matched the re-scan.

### Region facts (all proven, not inferred)

* `0x6D9148`, **1000 words / 2000 bytes** — *this is the reset/cleared half only*; the addressable
  variable space continues to `0x6DA8B8` (3000 words). See the INVALID banner above.
  `memset(&sjisText[908154], 0, 2000)` in `sub_409660` (= exactly this window), and
  `cmp ecx, 3E8h` / `imul edx, 3E8h` (1000) plus `mov ecx, 7D0h` (2000) in `sub_41BBE0`.
* The array symbol here is **`heap_vars`** (`unsigned short[1000]`, type applied). Note the cosmetic
  catch: the address falls inside a 910 KB byte-array item IDA auto-created at `sjisText` (`0x5FB5CE`)
  from a stale string reference, so `get_full_flags(0x6D9148)` is not a data head and *every* access keeps
  rendering as `sjisText[908154 + 2*i]`. Do not "fix" this by operand/item surgery across ~250 sites —
  just read `sjisText[908154+2*i]` as `heap_vars[i]`.
* Normal (indexed) access: `[reg*2 + 0x6D9148]`, 76 sites, in `sub_409660` (arithmetic / assignment
  opcodes: mov/add/sub/movsx on `g_scriptVars[eax*2]`) and `sub_40A050` (the interpreter main switch).
* On top of that, **56 slots are addressed by absolute address** (no index register) — i.e. the engine
  has hard-wired knowledge of specific script variables. Full dump: `font_work/hv_scan.txt`.

### Slots with engine (non-interpreter) accessors

| slot | address | accesses | accessing functions | notes |
|---|---|---|---|---|
| — | `0x6D9148` | 76 | sub_401630, sub_409660, sub_40A050, sub_419410, sub_419A90, sub_421BA0, sub_421C60, sub_421D60, sub_421E50, sub_421F60 | the array *base* (indexed access), not a slot |
| 940 | `0x6D98A0` | 1 | `sub_41BBE0` | address taken → part of a var-range scan loop in sub_41BBE0 |
| 942 | `0x6D98A4` | 2 | sub_40A050, `sub_41BBE0` | same range group |
| 952 | `0x6D98B8` | 1 | `sub_41BBE0` | same range group |
| 954 | `0x6D98BC` | 2 | **`Txt_DrawMainTextbox`**, sub_41BBE0 | passed as an id to `sub_439EF0` with mode 2/3 (`push 2` / `push 3`) — playback-style call |
| 955 | `0x6D98BE` | 2 | **`Txt_DrawMainTextbox`** | stored next to `timeGetTime()` stamps at `0x4F50E8` and `0x5F9068` ⇒ timing value |
| 972 | `0x6D98E0` | 1 | `sub_42CEB0` | read once |
| 975 | `0x6D98E6` | 1 | `sub_4170D0` | compared with 0 |
| **977** | `0x6D98EA` | 7 | sub_404130, sub_407E60, sub_410A00, sub_418F70 | **read-only from engine code**, written only through generic interpreter stores ⇒ a script-visible setting consumed by several screens |
| 980–986 | `0x6D98F0`–`0x6D98FC` | 1 each | `sub_4170D0` | seven consecutive slots read one-by-one in one function (a struct-of-7 the script writes; likely a window/rect or colour set) |
| **987** | `0x6D98FE` | 11 | sub_4119A0, sub_41BBE0 | compared with `2` in sub_4119A0, read ~10× in sub_41BBE0 ⇒ mode-like |
| 988 | `0x6D9900` | 3 | `sub_41BBE0` | |
| 989 | `0x6D9902` | 2 | `sub_41BBE0` | read in jumptable cases 2 / 12,13 of sub_41BBE0's switch |
| 991 | `0x6D9906` | 3 | sub_40A050, sub_419A90 | |
| 992 | `0x6D9908` | 4 | sub_40A050, sub_434D50 | compared against a register repeatedly in sub_434D50 |
| **993** | `0x6D990A` | 7 | **`Txt_ParseMarkers`**, sub_40A050, sub_41AF70 | written by the `%V` inline marker; reset by `sub_41AF70` (title/reset path, posts `"title"`) — named `g_varMarkerV` |
| **994** | `0x6D990C` | **49** | sub_404130, sub_407E60, sub_422A70, sub_423330, sub_423F80…, sub_43A8E0 (19 functions) | almost every access is `cmp word ptr …, bp` (== 0) in UI/config handlers ⇒ a global mode flag read by nearly every screen (skip/fast-forward-like semantics); **never written outside the interpreter** |
| 995 | `0x6D990E` | 7 | sub_40A050, sub_422060, sub_42C820, sub_42C8B0, sub_4357F0 | compared with 0 in three handlers; switch value in sub_4357F0 |
| **996** | `0x6D9910` | 21 | sub_40A050, sub_410A00, sub_4119A0, sub_42C940, sub_4348B0, sub_43BB60 | read + written by the interpreter and six engine routines ⇒ second-hottest slot after 994 |
| 997 | `0x6D9912` | 2 | `sub_40A050` | |
| 998 | `0x6D9914` | 1 | `sub_40A050` | write only |
| 999 | `0x6D9916` | 4 | sub_410A00, sub_4170D0, sub_41AF70 | written by the title/reset routine; compared with `6` in sub_4170D0 |

The remaining **30 slots** are touched only by the interpreter `sub_40A050` (hard-coded absolute reads
of mostly-low-numbered variables inside opcode handlers) — see `font_work/hv_scan.txt`.

### What is written into the IDB for this

* `heap_vars` = `unsigned short[1000]` at `0x6D9148` (name record + type; see the cosmetic caveat above).
* Element names: `g_varSpecialFontSel` (slot 979, font selection) and `g_varMarkerV` (slot 993, `%V`).
* Line comments on slots 954, 955, 977, 987, 994, 996, 999 summarising accessors (no invented semantics).
* Function comments on `sub_40A050`, `sub_409660`, `sub_41BBE0`, `sub_438CC0` and on the `sjisText` blob
  explaining the `sjisText[908154 + 2*i] == heap_vars[i]` identity.

### Why this matters for the translation/font work

Slot **979 is not unique** — the engine hard-codes per-slot behaviour in ~26 places, so any variable an
English patch wants to use for a new style switch should be chosen from slots that no engine code reads
(the 30 interpreter-only ones, or gaps such as 1–939 which are only reached through indexed access), and
conversely slots 977/987/994/996 must not be repurposed blindly because engine screens poll them.
