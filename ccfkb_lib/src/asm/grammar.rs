//! The grammar of the annotated assembly format: one nom parser per construct, plus the lexemes they
//! are built from. `assembly/README.md` is the normative text and every message here is one of its
//! rows, so changing a construct means editing the parser that owns the message.
//!
//! Positions: a parser runs on a slice of the raw line (as `str::lines` yields it), a failure carries
//! the slice it stopped at, and [`column_of`] turns that into the 1-based *character* column the
//! diagnostics and spans use. That is the only byte-offset-to-column conversion in the crate. Slices
//! are never re-based — blanks are skipped with [`space`], never with `trim` — so an offset is always
//! valid against the line, and one string rule (a `"…"` literal, honouring `\"`) governs every
//! scanner. Nothing here knows about a [`super::Diagnostic`]: this module produces [`AsmError`]s and
//! token structs, and `parse.rs` is the one place they become positioned diagnostics.

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1, take_while_m_n};
use nom::character::complete::{anychar, char};
use nom::combinator::{all_consuming, map, recognize, rest};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Offset, Parser};

use crate::opcodes::{Code, OpField, OpcodeSpecStatic, TLString};

use super::{is_jump_field, operand_labels};

/// The message a parser uses where the caller owns the wording: the line classifiers replace it with
/// the `unrecognized line` diagnostic, which only they can spell (they hold the line's text).
const UNRECOGNIZED: &str = "unrecognized line";

/// A parser failure: the message the format promises, and the slice it belongs to. [`column_of`]
/// turns `input` into the column a diagnostic is rendered with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AsmError<'a> {
	pub input: &'a str,
	pub message: String,
}

impl<'a> ParseError<&'a str> for AsmError<'a> {
	fn from_error_kind(input: &'a str, _: ErrorKind) -> Self {
		AsmError { input, message: UNRECOGNIZED.to_owned() }
	}

	/// Keeps the deeper parser's message: an `alt` branch must not replace a precise failure — the
	/// one the format's table pins — with a generic one.
	fn append(_: &'a str, _: ErrorKind, other: Self) -> Self {
		other
	}
}

/// This module's parser result.
pub(crate) type PResult<'a, O> = IResult<&'a str, O, AsmError<'a>>;

/// The 1-based character column of `slice` inside `line`. `slice` MUST point into `line`: every slice
/// a parser handles is a subslice of the raw line.
pub(crate) fn column_of(line: &str, slice: &str) -> usize {
	line[..line.offset(slice)].chars().count() + 1
}

/// Fails, positioned at `input`, with the message the format promises.
pub(crate) fn fail<'a, O>(input: &'a str, message: impl Into<String>) -> PResult<'a, O> {
	Err(nom::Err::Failure(AsmError { input, message: message.into() }))
}

/// A parser's failure as this module's error, dropping the leftover input. The caller keeps the error
/// itself when it needs the position ([`column_of`] on `input`), and [`value_of`] when it does not.
///
/// Every parser here is a complete parser, so the `Incomplete` case cannot arise.
pub(crate) fn finished<'a, T>(result: PResult<'a, T>) -> Result<T, AsmError<'a>> {
	match result {
		Ok((_, value)) => Ok(value),
		Err(nom::Err::Failure(it)) | Err(nom::Err::Error(it)) => Err(it),
		Err(nom::Err::Incomplete(_)) => {
			Err(AsmError { input: "", message: UNRECOGNIZED.to_owned() })
		}
	}
}

/// The value of a parser whose remainder and position the caller does not want: its output, or the
/// message the format promises.
pub(crate) fn value_of<T>(result: PResult<'_, T>) -> Result<T, String> {
	finished(result).map_err(|it| it.message)
}

// -- Lexemes ---------------------------------------------------------------------------------------

/// The insignificant whitespace of a line: exactly what `str::trim` strips, so the blanks around a
/// construct behave the same wherever they are written.
pub(crate) fn space(input: &str) -> PResult<'_, &str> {
	take_while(|c: char| c.is_whitespace()).parse(input)
}

/// A `"…"` literal, quotes included; `\"` does not close it and `\\` is data. Fails, consuming
/// nothing, when the quote is not closed on the line, which is what lets a scanner treat the rest of
/// the line as data instead.
pub(crate) fn string_token(input: &str) -> PResult<'_, &str> {
	recognize(preceded(
		char('"'),
		terminated(
			many0(alt((
				recognize(preceded(char('\\'), anychar)),
				take_while1(|c: char| c != '"' && c != '\\'),
			))),
			char('"'),
		),
	)).parse(input)
}

/// The rest of the line, from a `"` that is never closed: a malformed literal owns the rest of its
/// line, so what follows it is data (the line's error is the literal's, not a split's).
fn quoted_to_end(input: &str) -> PResult<'_, &str> {
	recognize(preceded(char('"'), rest)).parse(input)
}

/// As [`quoted_to_end`], from a `[` that is never closed.
fn bracketed_to_end(input: &str) -> PResult<'_, &str> {
	recognize(preceded(char('['), rest)).parse(input)
}

/// A `[ … ]` byte list, brackets included. Its items are bytes, not strings, so nothing inside is
/// special — a `"` or a `,` in there is data.
pub(crate) fn bracket_token(input: &str) -> PResult<'_, &str> {
	recognize(delimited(char('['), take_while(|c: char| c != ']'), char(']'))).parse(input)
}

/// The format's identifier: ASCII alphanumerics and `_`. Mnemonics, directive names and `label`
/// annotation values are all this token, and — as this parser has always accepted — so is a name
/// that starts with a digit.
pub(crate) fn word(input: &str) -> PResult<'_, &str> {
	take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_').parse(input)
}

/// A `;` comment: the text after the `;`, to the end of the line.
pub(crate) fn line_comment(input: &str) -> PResult<'_, &str> {
	preceded(char(';'), rest).parse(input)
}

/// A run of ordinary characters: not the terminator, not `"`, and — for a scanner that skips byte
/// lists — not `[`, so the literal alternatives get their turn.
fn plain_scan<'a>(
	terminator: fn(char) -> bool,
	brackets: bool,
) -> impl FnMut(&'a str) -> PResult<'a, &'a str> {
	move |input| {
		let ordinary = move |c: char| !terminator(c) && c != '"' && !(brackets && c == '[');
		take_while1(ordinary).parse(input)
	}
}

/// A run of input with no top-level `terminator`: the primitive behind the operand list, the
/// `label: value` split, the comment split and the inline-`#` guard.
///
/// `"…"` literals are skipped (honouring `\"`) and a literal that is never closed owns the rest of the
/// line. `brackets` widens the skip to `[ … ]` byte lists: the operand list and the `label: value`
/// split are bracket-aware, while the comment and inline-`#` scans are not — a `;` or `#` inside a
/// byte list is still top-level, exactly as the format and its printer have it.
///
/// Infallible: matching zero items is a match.
fn scan(terminator: fn(char) -> bool, brackets: bool, input: &str) -> PResult<'_, &str> {
	if brackets {
		recognize(many0(alt((
			string_token,
			quoted_to_end,
			bracket_token,
			bracketed_to_end,
			plain_scan(terminator, brackets),
		)))).parse(input)
	} else {
		recognize(many0(alt((
			string_token,
			quoted_to_end,
			plain_scan(terminator, brackets),
		)))).parse(input)
	}
}

/// [`scan`] as a parser value, for `separated_list0` and friends.
fn up_to<'a>(
	terminator: fn(char) -> bool,
	brackets: bool,
) -> impl FnMut(&'a str) -> PResult<'a, &'a str> {
	move |input| scan(terminator, brackets, input)
}

fn is_comma(c: char) -> bool {
	c == ','
}

fn is_colon(c: char) -> bool {
	c == ':'
}

fn is_semicolon(c: char) -> bool {
	c == ';'
}

fn is_hash(c: char) -> bool {
	c == '#'
}

// -- Values ----------------------------------------------------------------------------------------

/// A run of ASCII hex digits, as a parser value with this module's error type.
fn hex_run(input: &str) -> PResult<'_, &str> {
	take_while1(|c: char| c.is_ascii_hexdigit()).parse(input)
}

/// The `0x`/`0X` prefix of a hex literal, and what follows it.
fn hex_prefix(input: &str) -> PResult<'_, &str> {
	alt((tag("0x"), tag("0X"))).parse(input)
}

/// A `0x…` literal of exactly `digits` hex digits, all of it. The operand's label is named in both
/// messages, so this is the parser that owns the width and digit-count wording:
///
/// * not a `0x…` literal at all, or a literal of the wrong width, or a bare `0x` → `takes {width}`;
/// * a body with a character that is not a hex digit → `needs {digits} hex digits after "0x"`.
pub(crate) fn hex_value<'l, 'i>(
	label: &'l str,
	digits: usize,
	width: &'static str,
) -> impl Parser<&'i str, Output = u32, Error = AsmError<'i>> + 'l {
	move |input: &'i str| {
		let token = input.trim();
		let wrong_width = || {
			fail::<u32>(
				input,
				format!("operand \"{label}\" takes {width}, found \"{token}\""),
			)
		};
		let needs_digits = || {
			fail::<u32>(
				input,
				format!(
					"operand \"{label}\" needs {digits} hex digits after \"0x\", found \"{token}\""
				),
			)
		};
		let Ok((body, _)) = hex_prefix(token) else {
			return wrong_width();
		};
		if body.is_empty() {
			return wrong_width();
		}
		let Ok((leftover, body)) = hex_run(body) else {
			return needs_digits();
		};
		if !leftover.is_empty() {
			return needs_digits();
		}
		if body.len() != digits {
			return wrong_width();
		}
		Ok((leftover, u32::from_str_radix(body, 16).unwrap_or_default()))
	}
}

/// As [`hex_value`], but failing for a padded operand of the wrong length, which is its own row of the
/// message table.
pub(crate) fn padding_value<'l, 'i>(
	label: &'l str,
	size: u8,
) -> impl Parser<&'i str, Output = Vec<u8>, Error = AsmError<'i>> + 'l {
	move |input: &'i str| {
		if size == 1 {
			let (rest, byte) = hex_value(label, 2, "a 1-byte hex literal").parse(input)?;
			return Ok((rest, vec![byte as u8]));
		}
		let (rest, bytes) = byte_list_value(input)?;
		if bytes.len() != size as usize {
			return fail(
				input,
				format!(
					"operand \"{label}\" is {size} bytes in the opcode table, found {}",
					bytes.len()
				),
			);
		}
		Ok((rest, bytes))
	}
}

/// A `"…"` literal's value, unescaped: `\\ \" \n \r \t \xNN`, and every other character verbatim.
/// Fails on a literal that is not closed on the line, on a `\` escape the format does not define, and
/// on anything after the closing quote — a literal is the whole value.
pub(crate) fn string_value(input: &str) -> PResult<'_, String> {
	let Ok((rest, token)) = string_token(input) else {
		return fail(input, "unterminated string literal");
	};
	if !rest.is_empty() {
		return fail(input, "unterminated string literal");
	}
	match unescape(token) {
		Ok(text) => Ok((rest, text)),
		Err(message) => fail(input, message),
	}
}

/// Decodes the body of a `"…"` token: the escapes the printer writes, and every other character
/// verbatim.
fn unescape(token: &str) -> Result<String, String> {
	// A token this module produced is `"…"`: at least the two quotes, both one byte.
	let body = &token[1..token.len() - 1];
	let mut out = String::with_capacity(body.len());
	let mut chars = body.chars();
	while let Some(c) = chars.next() {
		if c != '\\' {
			out.push(c);
			continue;
		}
		// A token cannot end in a lone `\`: the tokenizer consumes escapes in pairs.
		let Some(escaped) = chars.next() else {
			return Err("unterminated string literal".to_owned());
		};
		match escaped {
			'\\' => out.push('\\'),
			'"' => out.push('"'),
			'n' => out.push('\n'),
			'r' => out.push('\r'),
			't' => out.push('\t'),
			'x' => {
				let hi = chars.next();
				let lo = chars.next();
				match (hi, lo) {
					(Some(hi), Some(lo)) if hi.is_ascii_hexdigit() && lo.is_ascii_hexdigit() => {
						let value = u8::from_str_radix(&format!("{hi}{lo}"), 16).unwrap_or_default();
						out.push(value as char);
					}
					_ => return Err("unknown escape \"\\x\" in a string literal".to_owned()),
				}
			}
			other => return Err(format!("unknown escape \"\\{other}\" in a string literal")),
		}
	}
	Ok(out)
}

/// One `[ … ]` byte list's brackets.
fn brackets(input: &str) -> PResult<'_, &str> {
	delimited(char('['), take_while(|c: char| c != ']'), char(']')).parse(input)
}

/// One `0xNN` item of a byte list.
fn byte_item(input: &str) -> PResult<'_, u8> {
	map(
		preceded(tag("0x"), take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit())),
		|it: &str| u8::from_str_radix(it, 16).unwrap_or_default(),
	).parse(input)
}

/// A `[ 0xNN, … ]` byte list, all of the value; `[ ]` is the empty list. The message names the text
/// the caller passed in, so the trailer, a padding operand and a choice record all get the one
/// wording the format documents.
pub(crate) fn byte_list_value(input: &str) -> PResult<'_, Vec<u8>> {
	let token = input.trim();
	let malformed = || fail::<Vec<u8>>(input, format!("malformed byte list \"{token}\""));
	let Ok((rest, inner)) = brackets(token) else {
		return malformed();
	};
	if !rest.is_empty() {
		return malformed();
	}
	let inner = inner.trim();
	if inner.is_empty() {
		return Ok((rest, Vec::new()));
	}
	let items = separated_list0(preceded(space, char(',')), preceded(space, byte_item));
	match all_consuming(items).parse(inner) {
		Ok((_, bytes)) => Ok((rest, bytes)),
		Err(_) => malformed(),
	}
}

/// A run of hex digits — all of it — as a number: the body of a `0x…` jump operand or of an `addr`
/// annotation, whose `0x` prefix is the caller's business. Fails on an empty run, on a body that
/// holds anything else, and on a body too wide for four bytes.
pub(crate) fn hex_digits(input: &str) -> PResult<'_, u32> {
	let token = input.trim();
	let Ok((rest, digits)) = hex_run(token) else {
		return fail(input, UNRECOGNIZED);
	};
	if !rest.is_empty() {
		return fail(input, UNRECOGNIZED);
	}
	match u32::from_str_radix(digits, 16) {
		Ok(value) => Ok((rest, value)),
		Err(_) => fail(input, UNRECOGNIZED),
	}
}

/// A jump operand's token: the label form the printer emits, a literal address, or a name to look up
/// in the file's labels. `None` is a token the driver reports as an undefined label.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum JumpTarget<'a> {
	/// `L_<hex>`: an address the printer knows is a jump destination.
	LabelAddress(u32),
	/// `0x<hex>`: an address the file spells literally.
	Address(u32),
	/// A name defined by a `# label` annotation.
	Name(&'a str),
}

/// The form of a jump token. Resolving an address to an instruction and a name to a label is the
/// driver's second pass, once every address is known.
pub(crate) fn jump_target(token: &str) -> Option<JumpTarget<'_>> {
	if let Some(hex) = token.strip_prefix("L_") {
		return hex_digits(hex).ok().map(|(_, value)| JumpTarget::LabelAddress(value));
	}
	if let Some(hex) = token.strip_prefix("0x").or_else(|| token.strip_prefix("0X")) {
		return hex_digits(hex).ok().map(|(_, value)| JumpTarget::Address(value));
	}
	Some(JumpTarget::Name(token))
}

// -- Lines -----------------------------------------------------------------------------------------

/// What a line's code half can be, before any of it is interpreted.
pub(crate) enum LineShape {
	Blank,
	Annotation,
	Directive,
	Record,
	Instruction,
	Bad,
}

/// Classifies a code half: blank; an annotation when the body starts with `#`, at any indentation;
/// otherwise by the byte count of the leading whitespace — two is a choice record, none is a
/// directive when the body starts with `.` and an instruction otherwise, and any other indentation is
/// a line no rule accepts.
pub(crate) fn classify(code: &str) -> LineShape {
	if code.trim().is_empty() {
		return LineShape::Blank;
	}
	let leading = code.len() - code.trim_start().len();
	let body = code.trim_start();
	if body.starts_with('#') {
		return LineShape::Annotation;
	}
	match leading {
		0 if body.starts_with('.') => LineShape::Directive,
		0 => LineShape::Instruction,
		2 => LineShape::Record,
		_ => LineShape::Bad,
	}
}

/// An annotation: the key (up to the first blank) and the value (the rest, trimmed). Annotations may
/// be indented, so leading blanks are skipped here.
pub(crate) fn annotation(input: &str) -> PResult<'_, (&str, &str)> {
	let (rest, _) = space(input)?;
	let (rest, _) = char('#').parse(rest)?;
	let (rest, _) = space(rest)?;
	let (rest, key) = take_while(|c: char| !c.is_whitespace()).parse(rest)?;
	Ok((rest, (key, rest.trim())))
}

/// A directive's name (after its `.`) and the text after it. The name is the longest of `.trailer`
/// and an identifier, so `.trailerX` reads as the directive `trailer` and the text `X` — which is
/// what makes the directive's own error message name the right text.
pub(crate) fn directive(input: &str) -> PResult<'_, (&str, &str)> {
	let (rest, _) = char('.').parse(input)?;
	let (rest, name) = alt((tag("trailer"), word)).parse(rest)?;
	Ok((rest, (name, rest)))
}

/// An instruction: its mnemonic and the operand region after it. Fails when the first token is not an
/// identifier, which is a line no rule accepts.
pub(crate) fn instruction(input: &str) -> PResult<'_, (&str, &str)> {
	let (rest, mnemonic) = word(input)?;
	if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
		return fail(input, UNRECOGNIZED);
	}
	let (rest, _) = space(rest)?;
	Ok((rest, (mnemonic, rest)))
}

/// A `# label` annotation's value: the format's identifier, all of it.
pub(crate) fn label_name(input: &str) -> PResult<'_, &str> {
	let (rest, name) = word(input)?;
	if !rest.is_empty() {
		return fail(input, UNRECOGNIZED);
	}
	Ok((rest, name))
}

/// The comma-separated operands of an instruction's region or of a choice record: a comma inside a
/// `"…"` literal or a `[ … ]` byte list does not separate, and a blank region has no operands.
///
/// Infallible: the separator always consumes a character, so the split cannot get stuck.
pub(crate) fn segments(input: &str) -> Vec<&str> {
	if input.trim().is_empty() {
		return Vec::new();
	}
	match separated_list0(char(','), up_to(is_comma, true)).parse(input) {
		Ok((_, segments)) => segments,
		Err(_) => Vec::new(),
	}
}

/// One operand's `label: value`: the tokens up to the first top-level `:`, and the text after it,
/// both trimmed. A `:` inside a literal or a byte list does not split, and an operand without a top
/// level `:` fails — which is how a line with an unparsable operand is recognized.
pub(crate) fn label_value(input: &str) -> PResult<'_, (&str, &str)> {
	let (rest, label) = scan(is_colon, true, input)?;
	let (rest, _) = char(':').parse(rest)?;
	Ok((rest, (label.trim(), rest.trim())))
}

/// The code half of a line and its trailing comment: a `;` inside a `"…"` literal is data. The
/// comment comes back trimmed, with the 1-based character column of its `;`.
pub(crate) fn code_and_comment(input: &str) -> (&str, Option<(&str, usize)>) {
	// The scan cannot fail: matching zero items is a match.
	let (rest, code) = scan(is_semicolon, false, input).unwrap_or((input, input));
	let Ok((_, after)) = line_comment(rest) else {
		return (code, None);
	};
	(code, Some((after.trim(), column_of(input, rest))))
}

/// Whether a line carries a `#` outside a `"…"` literal: an instruction or a choice record may not
/// spell an annotation inline.
pub(crate) fn has_top_level_hash(input: &str) -> bool {
	match scan(is_hash, false, input) {
		Ok((rest, _)) => rest.starts_with('#'),
		Err(_) => false,
	}
}

// -- One instruction line against one table row ----------------------------------------------------

/// One instruction line's decoded values, the label and slice of each operand (the driver turns the
/// slices into positioned spans) and the tokens of its jump operands, which the driver fills in once
/// every address is known.
pub(crate) struct Decoded<'a> {
	pub fields: Vec<OpField>,
	pub spans: Vec<(String, &'a str)>,
	pub jump_tokens: Vec<(usize, &'a str)>,
}

/// Parses one instruction line's operand region against one row of the opcode table. Which labels and
/// which values a region holds depends only on the row, so this is the parser that owns the
/// operand-count, operand-label/order and value messages.
///
/// Atomic: it decodes every operand or fails with the message that names the mismatch, so half a line
/// never escapes.
pub(crate) fn row_values<'a>(
	spec: &'static OpcodeSpecStatic,
) -> impl Parser<&'a str, Output = Decoded<'a>, Error = AsmError<'a>> + 'a {
	move |input: &'a str| {
		let labels = operand_labels(spec.operands);
		let operands = segments(input);
		if operands.len() != spec.operands.len() {
			return fail(
				input,
				format!(
					"{} (0x{:02X}) takes {} operands, found {}",
					spec.name,
					spec.opcode,
					spec.operands.len(),
					operands.len()
				),
			);
		}
		let order: Vec<String> = labels.iter().map(|it| format!("\"{it}\"")).collect();
		let mut fields = Vec::with_capacity(spec.layout.len());
		let mut spans = Vec::with_capacity(spec.layout.len());
		let mut jump_tokens = Vec::new();
		for (index, operand) in operands.iter().enumerate() {
			let operand = operand.trim();
			let label = &labels[index];
			// An operand with no top-level `:` is reported by its label check, which names it.
			let (found, value) = match label_value(operand) {
				Ok((_, it)) => it,
				Err(_) => (operand, ""),
			};
			if found != label {
				return fail(
					operand,
					format!(
						"operand {index} is \"{found}\"; {} (0x{:02X}) expects \"{label}\" (order: {})",
						spec.name,
						spec.opcode,
						order.join(", ")
					),
				);
			}
			spans.push((label.clone(), operand));
			let code = spec.layout[index];
			if matches!(code, Code::Choice) && index + 1 < spec.layout.len() {
				return fail(operand, "\"choices:\" must be the last operand on the line");
			}
			let field = match code {
				Code::Byte => OpField::Byte(
					hex_value(label, 2, "a 1-byte hex literal").parse(value)?.1 as u8,
				),
				Code::Word => OpField::Word(
					hex_value(label, 4, "a 2-byte hex literal").parse(value)?.1 as u16,
				),
				Code::DWord => {
					if is_jump_field(spec.opcode, index) {
						jump_tokens.push((index, value));
						OpField::DWord(0)
					} else {
						OpField::DWord(hex_value(label, 8, "a 4-byte hex literal").parse(value)?.1)
					}
				}
				Code::Str => OpField::String(TLString {
					raw: string_value(value)?.1,
					translation: None,
					notes: None,
				}),
				Code::Padding(size) => OpField::Padding(padding_value(label, size).parse(value)?.1),
				// The records arrive on the lines that follow the instruction; the value here names
				// nothing (the printer writes `choices:` with nothing after it).
				Code::Choice => OpField::Choice(vec![]),
			};
			fields.push(field);
		}
		Ok(("", Decoded { fields, spans, jump_tokens }))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn code_and_comment_is_quote_aware() {
		assert_eq!(code_and_comment("nop"), ("nop", None));
		assert_eq!(code_and_comment("; alone"), ("", Some(("alone", 1))));
		assert_eq!(
			code_and_comment("nop pad: 0x00 ; why"),
			("nop pad: 0x00 ", Some(("why", 15)))
		);
		// A `;` inside a `"…"` literal is data, and an escaped quote does not close the literal.
		assert_eq!(code_and_comment("  text: \"a; b\", pad: 0x00"), ("  text: \"a; b\", pad: 0x00", None));
		assert_eq!(code_and_comment("text: \"\\\";\""), ("text: \"\\\";\"", None));
		// A literal that is never closed owns the rest of the line.
		assert_eq!(code_and_comment("text: \"unclosed ; x"), ("text: \"unclosed ; x", None));
	}

	#[test]
	fn operands_split_on_top_level_commas() {
		let region = "text: \"a, b\", pad: 0x00";
		assert_eq!(segments(region).len(), 2, "{:?}", segments(region));
		let operands: Vec<(&str, &str)> = segments(region)
			.iter()
			.map(|it| value_of(label_value(it)).expect("a `label: value` operand"))
			.collect();
		assert_eq!(operands, vec![("text", "\"a, b\""), ("pad", "0x00")]);

		// A byte list keeps its own commas: the region holds one operand.
		assert_eq!(segments("trailer: [ 0x01, 0x02 ]").len(), 1);
		// A blank region has no operands, and an empty operand between commas is one.
		assert_eq!(segments(""), Vec::<&str>::new());
		assert_eq!(segments("0x00,").len(), 2);

		// Columns are character columns of the raw line, not byte offsets.
		let line = "nop テキスト: 0x00, pad: 0x777";
		let last = segments(line).pop().expect("two operands");
		let (label, value) = value_of(label_value(last)).expect("a `label: value` operand");
		assert_eq!(label, "pad");
		assert_eq!(column_of(line, value), 22);
		assert_eq!(column_of(line, last), 16, "the segment keeps the blank after the comma");
	}

	#[test]
	fn hex_value_messages() {
		fn value(input: &str) -> PResult<'_, u32> {
			hex_value("preset", 4, "a 2-byte hex literal").parse(input)
		}
		assert_eq!(value_of(value("0x0114")), Ok(0x0114));
		assert_eq!(value_of(value("0X0114")), Ok(0x0114));
		assert_eq!(
			value_of(value("0x777")),
			Err("operand \"preset\" takes a 2-byte hex literal, found \"0x777\"".to_owned())
		);
		assert_eq!(
			value_of(value("777")),
			Err("operand \"preset\" takes a 2-byte hex literal, found \"777\"".to_owned())
		);
		assert_eq!(
			value_of(value("0x")),
			Err("operand \"preset\" takes a 2-byte hex literal, found \"0x\"".to_owned())
		);
		assert_eq!(
			value_of(value("0xZZ")),
			Err("operand \"preset\" needs 4 hex digits after \"0x\", found \"0xZZ\"".to_owned())
		);
		assert_eq!(
			value_of(value("0x0114junk")),
			Err("operand \"preset\" needs 4 hex digits after \"0x\", found \"0x0114junk\"".to_owned())
		);
		// The failure points at the value token, not at the operand's label.
		match value("0x777") {
			Err(nom::Err::Failure(it)) => assert_eq!(it.input, "0x777"),
			other => panic!("expected a positioned failure, got {other:?}"),
		}
	}

	#[test]
	fn string_value_escapes() {
		let value = |input: &str| value_of(string_value(input));
		assert_eq!(value("\"\""), Ok(String::new()));
		assert_eq!(value("\"a\\\\b\""), Ok("a\\b".to_owned()));
		assert_eq!(value("\"\\\"\""), Ok("\"".to_owned()));
		assert_eq!(value("\"\\n\\r\\t\""), Ok("\n\r\t".to_owned()));
		assert_eq!(value("\"\\x1F\""), Ok("\u{1F}".to_owned()));
		assert_eq!(value("\"あ, 、\""), Ok("あ, 、".to_owned()));
		assert_eq!(
			value("\"abc"),
			Err("unterminated string literal".to_owned())
		);
		assert_eq!(value("abc"), Err("unterminated string literal".to_owned()));
		assert_eq!(
			value("\"x\" junk"),
			Err("unterminated string literal".to_owned()),
			"a literal is the whole value"
		);
		assert_eq!(
			value("\"\\q\""),
			Err("unknown escape \"\\q\" in a string literal".to_owned())
		);
		assert_eq!(
			value("\"\\x\""),
			Err("unknown escape \"\\x\" in a string literal".to_owned())
		);
	}

	#[test]
	fn byte_list_and_padding() {
		assert_eq!(value_of(byte_list_value("[ ]")), Ok(vec![]));
		assert_eq!(value_of(byte_list_value("[ 0x00, 0x01 ]")), Ok(vec![0x00, 0x01]));
		assert_eq!(value_of(byte_list_value("[0x00,0x01]")), Ok(vec![0x00, 0x01]));
		for malformed in ["[ 0x00, 0x01", "[ 0x00, 0x01 ] junk", "[ 0X00 ]", "[ 0x000 ]", "[ 0x00, ]"] {
			assert_eq!(
				value_of(byte_list_value(malformed)),
				Err(format!("malformed byte list \"{malformed}\"")),
				"{malformed}"
			);
		}

		let padding = |size: u8, input: &str| value_of(padding_value("pad", size).parse(input));
		assert_eq!(padding(1, "0x00"), Ok(vec![0x00]));
		assert_eq!(
			padding(1, "[ 0x00 ]"),
			Err("operand \"pad\" takes a 1-byte hex literal, found \"[ 0x00 ]\"".to_owned())
		);
		assert_eq!(
			padding(1, "0x000"),
			Err("operand \"pad\" takes a 1-byte hex literal, found \"0x000\"".to_owned())
		);
		assert_eq!(padding(2, "[ 0x00, 0x01 ]"), Ok(vec![0x00, 0x01]));
		assert_eq!(
			padding(2, "[ 0x00 ]"),
			Err("operand \"pad\" is 2 bytes in the opcode table, found 1".to_owned())
		);
		assert_eq!(
			padding(2, "0x00"),
			Err("malformed byte list \"0x00\"".to_owned())
		);
	}

	#[test]
	fn line_shapes() {
		assert!(matches!(classify(""), LineShape::Blank));
		assert!(matches!(classify("   "), LineShape::Blank));
		assert!(matches!(classify("# addr 0x00000000"), LineShape::Annotation));
		assert!(matches!(classify("  # indented"), LineShape::Annotation));
		assert!(matches!(classify(".trailer [ ]"), LineShape::Directive));
		assert!(matches!(classify("  arg1: 0x0001, text: \"a\""), LineShape::Record));
		assert!(matches!(classify("nop"), LineShape::Instruction));
		assert!(matches!(classify(" nop"), LineShape::Bad), "one blank is no indent");
		assert!(matches!(classify("   nop"), LineShape::Bad), "three blanks are no indent");
		assert!(
			matches!(classify("  .trailer [ ]"), LineShape::Record),
			"two blanks make a record of anything"
		);

		assert_eq!(value_of(annotation("# addr 0x00000000")), Ok(("addr", "0x00000000")));
		assert_eq!(value_of(annotation("#yields")), Ok(("yields", "")));
		assert_eq!(value_of(annotation("   # translation \"x y\"")), Ok(("translation", "\"x y\"")));
		assert_eq!(value_of(annotation("#")), Ok(("", "")));
		assert_eq!(value_of(annotation("#  spaced   out  ")), Ok(("spaced", "out")));

		assert_eq!(value_of(directive(".trailer [ ]")), Ok(("trailer", " [ ]")));
		assert_eq!(value_of(directive(".trailerX")), Ok(("trailer", "X")));
		assert_eq!(value_of(directive(".foo bar")), Ok(("foo", " bar")));
		assert!(directive(".").is_err(), "a lone dot is not a directive");

		assert_eq!(value_of(instruction("nop")), Ok(("nop", "")));
		assert_eq!(value_of(instruction("nop   pad: 0x00")), Ok(("nop", "pad: 0x00")));
		assert_eq!(value_of(instruction("nop\tpad: 0x00")), Ok(("nop", "pad: 0x00")));
		assert_eq!(value_of(instruction("1nop")), Ok(("1nop", "")), "a digit may lead");
		assert!(instruction("nope x!").is_ok(), "the token ends at the blank");
		assert!(instruction("nope! x").is_err(), "`!` is no mnemonic");

		assert_eq!(value_of(label_name("main")), Ok("main"));
		assert_eq!(value_of(label_name("L_0000009D")), Ok("L_0000009D"));
		assert!(label_name("").is_err());
		assert!(label_name("two words").is_err());
	}

	#[test]
	fn hex_digits_and_jump_targets() {
		assert_eq!(value_of(hex_digits("0000009D")), Ok(0x9D));
		assert!(hex_digits("").is_err(), "an empty run is no number");
		assert!(hex_digits("9DZ").is_err());
		assert!(hex_digits("FFFFFFFFF").is_err(), "five bytes do not fit");

		assert_eq!(jump_target("L_0000009D"), Some(JumpTarget::LabelAddress(0x9D)));
		assert_eq!(jump_target("0x0000009D"), Some(JumpTarget::Address(0x9D)));
		assert_eq!(jump_target("0X0000009D"), Some(JumpTarget::Address(0x9D)));
		assert_eq!(jump_target("main"), Some(JumpTarget::Name("main")));
		assert_eq!(jump_target("L_zz"), None);
		assert_eq!(jump_target("0x"), None);
	}

	#[test]
	fn a_hash_inside_a_literal_is_data() {
		assert!(!has_top_level_hash("nop pad: 0x00"));
		assert!(has_top_level_hash("nop pad: 0x00 # trailing"));
		assert!(!has_top_level_hash("text: \"a # b\""));
		assert!(
			!has_top_level_hash("text: \"unclosed # b"),
			"an unclosed literal still owns what follows it"
		);
		assert!(has_top_level_hash("pad: [ 0x0# ]"), "a byte list is no literal");
		assert!(has_top_level_hash("  arg1: 0x0001 # trailing"));
	}
}
