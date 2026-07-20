#!/usr/bin/env python3
"""Validate the CORE-003 signing guides and their local Markdown links."""

from __future__ import annotations

import re
import sys
import unicodedata
from dataclasses import dataclass
from html import unescape
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
SIGNING_DIR = ROOT / "docs" / "signing"
WORKFLOW = ROOT / ".github" / "workflows" / "main.yml"

REQUIRED_GUIDES = (
    "babylon.md",
    "bitcoin.md",
    "dlc.md",
    "liquid.md",
    "rgb.md",
    "stacks.md",
)

COMMON_SECTIONS = (
    "scope and current support status",
    "canonical target and ucs boundary",
    "participant ownership",
    "sequence and ownership",
    "required inputs",
    "required outputs",
    "verification and finality boundary",
    "retry versus terminal semantics",
    "fail-closed boundaries",
    "known gaps and unsupported behavior",
    "source references",
)

PARTICIPANTS = (
    "Core",
    "conxius-enclave-sdk",
    "Gateway",
    "Nexus",
    "Wallet",
)

# These anchors are intentionally kept scoped to the section where they are
# meaningful.  A guide cannot satisfy a boundary requirement by mentioning the
# same word in an unrelated section.
COMMON_SECTION_ANCHORS = {
    "canonical target and ucs boundary": (
        "SignRequest",
        "SignResponse",
        "SignerCapabilities",
    ),
    "participant ownership": (
        "conxius-enclave-sdk",
        "Gateway",
        "Nexus",
        "Wallet",
    ),
    "verification and finality boundary": ("ProtocolVerifier",),
    "retry versus terminal semantics": (
        "operational retry classification is downstream policy",
    ),
    "source references": (
        "conxian-gateway",
        "conxian-nexus",
        "conxius-wallet",
    ),
}

CHAIN_SECTION_ANCHORS = {
    "bitcoin.md": {
        "canonical target and ucs boundary": (
            "BIP-341",
            "BIP-342",
            "BIP-110",
            "256",
            "83",
            "34",
            "Bip110Compliance::default()",
            "PSBT",
            "Taproot",
        ),
        "verification and finality boundary": ("BIP-322",),
    },
    "stacks.md": {
        "scope and current support status": (
            "SBTCIntent",
            "SBTCState",
            "pilot",
            "hardcoded",
            "ContractBridge",
        ),
        "canonical target and ucs boundary": ("Chain::Stacks",),
        "verification and finality boundary": ("Bitcoin finality", "BIP-110"),
    },
    "babylon.md": {
        "scope and current support status": (
            "StakingIntent",
            "EOTS",
            "structural",
        ),
        "canonical target and ucs boundary": (
            "Chain::Babylon",
            "BabylonAdapter::chain()",
            "Chain::Bitcoin",
        ),
        "verification and finality boundary": (
            "EOTS",
            "structural",
            "BTC headers",
        ),
    },
    "liquid.md": {
        "scope and current support status": (
            "peg-in",
            "peg-out",
            "Elements",
            "confidential",
            "LiquidAdapter",
            "structural",
        ),
        "canonical target and ucs boundary": ("Chain::Liquid",),
    },
    "rgb.md": {
        "scope and current support status": (
            "RGBAdapter",
            "Shadow",
            "non-enforcing",
        ),
        "canonical target and ucs boundary": ("Chain::RGB", "Bitcoin anchor"),
        "sequence and ownership": ("validate_transition", "verify_seal"),
    },
    "dlc.md": {
        "scope and current support status": (
            "DlcIntent",
            "oracle",
            "verify_oracle_attestation",
            "verify_execution",
            "compatibility-only",
        ),
        "sequence and ownership": ("funding", "refund", "CET"),
    },
}

# This is deliberately a small, literal source map.  It is not a Rust parser;
# it only catches missing files and obvious symbol/file refactors that would
# make the guides point at stale Core contracts.
SOURCE_ANCHORS = {
    "src/signing.rs": (
        "pub struct SignRequest",
        "pub struct SignResponse",
        "pub struct SignerCapabilities",
        "pub trait UniversalChainSigner",
    ),
    "src/verifier.rs": (
        "pub struct ProtocolVerifier",
        "pub trait ProtocolVerifierBackend",
    ),
    "src/control_model/trust.rs": (
        "pub enum Chain",
        "pub struct Bip110Compliance",
    ),
    "src/control_model/bip110.rs": (
        "pub struct Bip110TransactionShape",
        "pub fn validate(&self)",
        "pub fn validate_transaction",
    ),
    "src/adapters/mod.rs": ("pub trait UniversalChainAdapter",),
    "src/bitcoin/bip322.rs": (
        "pub struct Bip322Message",
        "pub struct Bip322Bridge",
        "pub fn verify_message",
    ),
    "src/bitcoin/liquid_adapter.rs": (
        "pub struct LiquidAdapter",
        "Chain::Liquid",
    ),
    "src/babylon/mod.rs": (
        "pub struct StakingIntent",
        "pub struct BabylonAdapter",
        "fn chain",
        "verify_state_proof",
    ),
    "src/stacks/mod.rs": (
        "pub enum SBTCState",
        "pub struct SBTCIntent",
        "pub trait StacksAdapter",
        "pub struct SBTCBridge",
        "pub struct StacksNakamoto",
        "verify_bitcoin_finality",
    ),
    "src/contract_bridge.rs": (
        "pub struct ContractBridge",
        "pub fn create_signed_call",
    ),
    "src/rgb/mod.rs": (
        "pub enum RGBExecutionMode",
        "pub trait RGBAdapter",
        "validate_transition",
        "verify_seal",
    ),
    "src/protocol/dlc.rs": (
        "pub struct DlcIntent",
        "create_intent",
        "verify_oracle_attestation",
        "verify_execution",
    ),
}

HEADING_RE = re.compile(r"^(#{1,6})[ \t]+(.+?)[ \t]*#*[ \t]*$")
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)\n]+)\)")
SCHEME_RE = re.compile(r"^[A-Za-z][A-Za-z0-9+.-]*:")
TABLE_CELL_RE = re.compile(r"^:?-{3,}:?$")
LIST_ITEM_RE = re.compile(r"^\s*(?:[-+*]|\d+[.)])(?:[ \t]+(.*))?$")
WORKFLOW_COMMAND_RE = re.compile(
    r"^python(?:3(?:\.\d+)?)?[ \t]+scripts/verify_signing_docs\.py(?:[ \t]|$)"
)

FAIL_CLOSED_RE = re.compile(
    r"\b(?:stop|reject(?:ed|s)?|block(?:ed|s)?|must[ \t]+not|do[ \t]+not|never|"
    r"fail[ \t-]*(?:closed|closedly))\b",
    re.IGNORECASE,
)
RETRY_LABEL_RE = re.compile(r"\b(?:retryable|waitable|retry|wait)\b", re.IGNORECASE)
TERMINAL_LABEL_RE = re.compile(r"\bterminal\b", re.IGNORECASE)
RETRY_CONDITION_RE = re.compile(
    r"\b(?:temporar(?:y|ily)|delayed|pending|unavailable|confirmation|"
    r"observation|provider|wait(?:ing)?|arriv(?:e|ed|al)|open)\b",
    re.IGNORECASE,
)
TERMINAL_ATTEMPT_RE = re.compile(
    r"\b(?:invalid|malformed|unsupported|mismatch|failed|failure|reject(?:ed)?|"
    r"expired|stale|payload|target|capability|response|request|intent|address|"
    r"transaction|operation|signer|oracle)\b",
    re.IGNORECASE,
)
TERMINAL_EVIDENCE_RE = re.compile(
    r"\b(?:proof|evidence|verif(?:y|ied|ication)|finality|policy|binding|"
    r"signature|attestation|quorum|precondition|validation)\b",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class Heading:
    line_number: int
    level: int
    title: str


@dataclass(frozen=True)
class Section:
    title: str
    level: int
    body: str


@dataclass(frozen=True)
class MarkdownTable:
    line_number: int
    header: tuple[str, ...]
    separator: tuple[str, ...]
    rows: tuple[tuple[str, ...], ...]


def normalized(value: str) -> str:
    """Normalize heading or prose text without enforcing exact formatting."""

    return " ".join(unescape(value).casefold().split())


def heading_slug(value: str) -> str:
    """Return a deterministic GitHub-like slug for a heading/fragment."""

    value = unicodedata.normalize("NFKC", unescape(unquote(value))).casefold()
    kept = [
        character
        for character in value
        if character.isalnum()
        or character in {" ", "-", "_"}
        or character.isspace()
    ]
    slug = "".join(kept)
    slug = re.sub(r"\s+", "-", slug)
    return slug.strip("-")


def iter_headings(text: str) -> tuple[Heading, ...]:
    """Return Markdown headings outside fenced code blocks in source order."""

    headings: list[Heading] = []
    fence_character: str | None = None
    fence_re = re.compile(r"^\s*(`{3,}|~{3,})")

    for line_number, line in enumerate(text.splitlines()):
        fence = fence_re.match(line)
        if fence:
            marker = fence.group(1)[0]
            if fence_character is None:
                fence_character = marker
            elif marker == fence_character:
                fence_character = None
            continue
        if fence_character is not None:
            continue

        match = HEADING_RE.match(line)
        if match:
            headings.append(Heading(line_number, len(match.group(1)), match.group(2).strip()))

    return tuple(headings)


def parse_sections(text: str) -> tuple[Section, ...]:
    """Parse sections using heading levels so nested headings stay in parents."""

    lines = text.splitlines()
    headings = iter_headings(text)
    sections: list[Section] = []

    for index, heading in enumerate(headings):
        end_line = len(lines)
        for following in headings[index + 1 :]:
            if following.level <= heading.level:
                end_line = following.line_number
                break
        sections.append(
            Section(
                title=heading.title,
                level=heading.level,
                body="\n".join(lines[heading.line_number + 1 : end_line]),
            )
        )

    return tuple(sections)


def section_lookup(text: str) -> dict[str, Section]:
    """Index the first occurrence of each normalized heading deterministically."""

    sections: dict[str, Section] = {}
    for section in parse_sections(text):
        sections.setdefault(normalized(section.title), section)
    return sections


def heading_slugs(text: str) -> set[str]:
    """Return GitHub-like heading slugs, including duplicate suffixes."""

    used: set[str] = set()
    next_suffix: dict[str, int] = {}
    for heading in iter_headings(text):
        base = heading_slug(heading.title)
        if not base:
            continue

        suffix = next_suffix.get(base, 0)
        candidate = base if suffix == 0 else f"{base}-{suffix}"
        while candidate in used:
            suffix += 1
            candidate = f"{base}-{suffix}"
        next_suffix[base] = suffix + 1
        used.add(candidate)

    return used


def section_has_body(section: Section) -> bool:
    """Require actual section content, not just a nested heading/whitespace."""

    return any(line.strip() and not HEADING_RE.match(line) for line in section.body.splitlines())


def _is_escaped(text: str, index: int) -> bool:
    backslashes = 0
    index -= 1
    while index >= 0 and text[index] == "\\":
        backslashes += 1
        index -= 1
    return backslashes % 2 == 1


def split_table_row(line: str) -> tuple[str, ...] | None:
    """Split a Markdown table row without treating escaped/code pipes as delimiters."""

    text = line.strip()
    if not text or "|" not in text:
        return None
    if text.startswith("|"):
        text = text[1:]
    if text.endswith("|") and not _is_escaped(text, len(text) - 1):
        text = text[:-1]

    cells: list[str] = []
    current: list[str] = []
    code_ticks = 0
    index = 0
    while index < len(text):
        character = text[index]
        if character == "`":
            run_end = index
            while run_end < len(text) and text[run_end] == "`":
                run_end += 1
            run_length = run_end - index
            if code_ticks == 0:
                code_ticks = run_length
            elif run_length == code_ticks:
                code_ticks = 0
            current.extend(text[index:run_end])
            index = run_end
            continue
        if character == "|" and code_ticks == 0:
            if _is_escaped(text, index):
                if current and current[-1] == "\\":
                    current.pop()
                current.append("|")
            else:
                cells.append("".join(current).strip())
                current = []
        else:
            current.append(character)
        index += 1
    cells.append("".join(current).strip())

    return tuple(cells) if len(cells) >= 2 else None


def is_table_separator(row: tuple[str, ...] | None) -> bool:
    return bool(row and all(TABLE_CELL_RE.fullmatch(cell.strip()) for cell in row))


def parse_markdown_tables(body: str) -> tuple[MarkdownTable, ...]:
    """Parse contiguous Markdown tables with a structural header/separator pair."""

    lines = body.splitlines()
    tables: list[MarkdownTable] = []
    index = 0
    while index < len(lines) - 1:
        header = split_table_row(lines[index])
        separator = split_table_row(lines[index + 1])
        if header is None or not is_table_separator(separator):
            index += 1
            continue

        rows: list[tuple[str, ...]] = []
        row_index = index + 2
        while row_index < len(lines):
            line = lines[row_index]
            if not line.strip():
                break
            row = split_table_row(line)
            if row is None:
                break
            rows.append(row)
            row_index += 1
        tables.append(
            MarkdownTable(
                line_number=index + 1,
                header=header,
                separator=separator or (),
                rows=tuple(rows),
            )
        )
        index = max(row_index, index + 1)

    return tuple(tables)


def normalized_cell(value: str) -> str:
    """Remove lightweight Markdown wrappers before matching a table label."""

    value = re.sub(r"`+", "", unescape(value))
    value = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", value)
    return normalized(value)


def bounded_anchor_present(text: str, anchor: str) -> bool:
    """Match an anchor as a token/phrase rather than an unrestricted substring."""

    words = anchor.split()
    pattern = r"(?<![A-Za-z0-9_])" + r"[ \t\r\n]+".join(
        re.escape(word) for word in words
    ) + r"(?![A-Za-z0-9_])"
    return re.search(pattern, text, re.IGNORECASE) is not None


def cell_has_label(cell: str, label: str) -> bool:
    value = normalized_cell(cell)
    target = normalized(label)
    if value == target:
        return True
    pattern = (
        r"(?<![A-Za-z0-9_-])"
        + re.escape(target)
        + r"(?![A-Za-z0-9_-])"
    )
    return re.search(pattern, value, re.IGNORECASE) is not None


def list_items(body: str) -> tuple[str, ...]:
    """Collect list entries, including indented continuation lines."""

    items: list[str] = []
    current: list[str] | None = None
    for line in body.splitlines():
        match = LIST_ITEM_RE.match(line)
        if match:
            if current is not None:
                items.append(" ".join(part for part in current if part).strip())
            current = [(match.group(1) or "").strip()]
            continue

        if current is None:
            continue
        if not line.strip() or HEADING_RE.match(line) or split_table_row(line):
            if not line.strip():
                items.append(" ".join(part for part in current if part).strip())
                current = None
            continue
        current.append(line.strip())

    if current is not None:
        items.append(" ".join(part for part in current if part).strip())
    return tuple(items)


def is_substantive_text(value: str) -> bool:
    plain = re.sub(r"[*_`#]", "", unescape(value)).strip()
    return len(re.findall(r"[A-Za-z0-9]", plain)) >= 8


def has_substantive_entries(body: str, minimum: int = 2) -> bool:
    items = list_items(body)
    if items:
        return len(items) >= minimum and all(is_substantive_text(item) for item in items)

    for table in parse_markdown_tables(body):
        if len(table.rows) >= minimum and all(
            all(is_substantive_text(cell) for cell in row) for row in table.rows
        ):
            return True
    return False


def participant_header(header: tuple[str, ...]) -> bool:
    if len(header) < 3:
        return False
    first = normalized(header[0])
    ownership_columns = sum(
        bool(re.search(r"\b(?:own|responsib|scope|boundary)\w*\b", normalized(cell)))
        for cell in header[1:]
    )
    return bool(
        re.search(r"\b(?:participant|actor|role|owner)\b", first)
        and ownership_columns >= 2
    )


def validate_participant_table(body: str) -> list[str]:
    tables = parse_markdown_tables(body)
    candidates = [
        table
        for table in tables
        if len(table.header) == len(table.separator) and participant_header(table.header)
    ]
    if not candidates:
        return ["participant ownership has no recognizable structural table"]

    table = candidates[0]
    errors: list[str] = []
    for participant in PARTICIPANTS:
        matches = [row for row in table.rows if row and cell_has_label(row[0], participant)]
        if not matches:
            errors.append(f"participant ownership table missing row: {participant}")
            continue
        row = matches[0]
        if len(row) != len(table.header):
            errors.append(f"participant ownership row has wrong column count: {participant}")
            continue
        if any(not cell.strip() for cell in row[1:]):
            errors.append(f"participant ownership row has empty ownership cell: {participant}")
    return errors


def sequence_header(header: tuple[str, ...]) -> bool:
    if len(header) != 6:
        return False
    values = tuple(normalized(cell) for cell in header)
    return (
        bool(re.search(r"\bstep\b", values[0]))
        and bool(re.search(r"\bowner\b", values[1]))
        and "input" in values[2]
        and "evidence" in values[2]
        and "core" in values[3]
        and bool(re.search(r"\b(?:contract|boundary)\b", values[3]))
        and "output" in values[4]
        and "stop" in values[5]
        and bool(re.search(r"\b(?:condition|criteria)\b", values[5]))
    )


def validate_sequence_table(body: str) -> list[str]:
    tables = parse_markdown_tables(body)
    candidates = [
        table
        for table in tables
        if len(table.header) == len(table.separator) and sequence_header(table.header)
    ]
    if not candidates:
        return [
            "sequence and ownership has no table with Step, Owner, input/evidence, "
            "Core contract/boundary, Output, and stop-condition columns"
        ]

    table = candidates[0]
    errors: list[str] = []
    if len(table.rows) < 2:
        errors.append("sequence and ownership needs at least two data rows")
    for row_number, row in enumerate(table.rows, start=1):
        if len(row) != len(table.header):
            errors.append(f"sequence and ownership data row {row_number} has wrong column count")
            continue
        if any(not cell.strip() for cell in row):
            errors.append(f"sequence and ownership data row {row_number} has an empty column")
        if not re.match(r"^\s*\d+\b", row[0]):
            errors.append(f"sequence and ownership data row {row_number} has no step number")
    return errors


def validate_retry_semantics(body: str) -> list[str]:
    items = list_items(body)
    retry_items = [item for item in items if RETRY_LABEL_RE.search(item)]
    terminal_items = [item for item in items if TERMINAL_LABEL_RE.search(item)]
    errors: list[str] = []
    if not retry_items:
        errors.append("retry versus terminal semantics has no retryable/waitable rule")
    elif not any(RETRY_CONDITION_RE.search(item) for item in retry_items):
        errors.append("retry versus terminal semantics has no operational retry condition")
    if not terminal_items:
        errors.append("retry versus terminal semantics has no terminal rule")
    else:
        terminal_text = " ".join(terminal_items)
        if not TERMINAL_ATTEMPT_RE.search(terminal_text):
            errors.append("terminal semantics has no current-attempt/input failure")
        if not TERMINAL_EVIDENCE_RE.search(terminal_text):
            errors.append("terminal semantics has no evidence/verification failure")
    return errors


def validate_fail_closed(body: str) -> list[str]:
    rules = [item for item in list_items(body) if FAIL_CLOSED_RE.search(item)]
    if len(rules) < 2:
        return [
            "fail-closed boundaries needs at least two explicit stop/reject/block/must-not rules"
        ]
    return []


def validate_known_gaps(body: str) -> list[str]:
    if not has_substantive_entries(body):
        return ["known gaps and unsupported behavior has no substantive structured entries"]
    return []


def validate_source_references(body: str) -> list[str]:
    items = list_items(body)
    links = tuple(LINK_RE.finditer(body))
    if len(items) < 2 or len(links) < 2:
        return [
            "source references needs multiple structured entries with Markdown links"
        ]
    return []


def markdown_files_to_check() -> tuple[Path, ...]:
    files = sorted(SIGNING_DIR.glob("*.md"))
    readme = ROOT / "README.md"
    if readme.exists():
        files.append(readme)
    return tuple(files)


def check_local_links(path: Path, text: str) -> list[str]:
    path = path.resolve()
    errors: list[str] = []
    for match in LINK_RE.finditer(text):
        destination = match.group(1).strip()
        if destination.startswith("<") and ">" in destination:
            destination = destination[1 : destination.index(">")]
        else:
            destination = destination.split(maxsplit=1)[0]

        # Absolute URLs and protocol-relative links are outside the local
        # target check.  A fragment-only link is deliberately checked against
        # the current Markdown file rather than skipped.
        if not destination or destination.startswith("//"):
            continue
        if SCHEME_RE.match(destination):
            continue

        parsed = urlsplit(destination)
        target = unquote(parsed.path)
        resolved = (path.parent / target).resolve() if target else path.resolve()
        try:
            resolved.relative_to(ROOT)
        except ValueError:
            errors.append(f"{path.relative_to(ROOT)}: link escapes repository: {destination}")
            continue
        if not resolved.exists():
            errors.append(f"{path.relative_to(ROOT)}: missing local link target: {destination}")
            continue

        fragment = unquote(parsed.fragment)
        if fragment and resolved.suffix.casefold() in {".md", ".markdown"}:
            if heading_slug(fragment) not in heading_slugs(resolved.read_text(encoding="utf-8")):
                errors.append(
                    f"{path.relative_to(ROOT)}: missing local link fragment: {destination}"
                )
    return errors


def check_source_anchors() -> list[str]:
    errors: list[str] = []
    for relative, anchors in SOURCE_ANCHORS.items():
        path = ROOT / relative
        if not path.is_file():
            errors.append(f"source anchor file is missing: {relative}")
            continue
        source = path.read_text(encoding="utf-8")
        for anchor in anchors:
            if anchor not in source:
                errors.append(f"source anchor missing from {relative}: {anchor}")
    return errors


def _active_line(line: str) -> bool:
    return bool(line.strip()) and not line.lstrip().startswith("#")


def _indent_width(line: str) -> int:
    return len(line) - len(line.lstrip(" \t"))


def _workflow_command(value: str) -> bool:
    value = value.strip()
    return not value.startswith("#") and WORKFLOW_COMMAND_RE.match(value) is not None


def workflow_has_validator_invocation(text: str) -> bool:
    """Accept only an active direct run or command inside an active run block."""

    lines = text.splitlines()
    run_re = re.compile(r"^(?P<indent>[ \t]*)(?:-[ \t]*)?run:[ \t]*(?P<value>.*)$")
    for index, line in enumerate(lines):
        if not _active_line(line):
            continue
        match = run_re.match(line)
        if not match:
            continue

        value = match.group("value")
        if _workflow_command(value):
            return True
        if not value.strip().startswith("|"):
            continue

        run_indent = _indent_width(line)
        for child in lines[index + 1 :]:
            if not child.strip():
                continue
            if not _active_line(child):
                continue
            if _indent_width(child) <= run_indent:
                break
            if _workflow_command(child.strip()):
                return True
    return False


def check_guide(filename: str) -> list[str]:
    path = SIGNING_DIR / filename
    errors: list[str] = []
    relative = path.relative_to(ROOT)
    if not path.is_file():
        return [f"missing required guide: {relative}"]

    text = path.read_text(encoding="utf-8")
    sections = section_lookup(text)
    for section_name in COMMON_SECTIONS:
        section_key = normalized(section_name)
        current = sections.get(section_key)
        if current is None:
            errors.append(f"{relative}: missing section: {section_name}")
            continue
        if not section_has_body(current):
            errors.append(f"{relative}: empty section: {section_name}")
            continue

        if section_key == normalized("participant ownership"):
            errors.extend(
                f"{relative}: {error}" for error in validate_participant_table(current.body)
            )
        elif section_key == normalized("sequence and ownership"):
            errors.extend(
                f"{relative}: {error}" for error in validate_sequence_table(current.body)
            )
        elif section_key in {
            normalized("required inputs"),
            normalized("required outputs"),
        }:
            if not has_substantive_entries(current.body):
                errors.append(f"{relative}: {section_name} has no substantive entries")
        elif section_key == normalized("retry versus terminal semantics"):
            errors.extend(
                f"{relative}: {error}" for error in validate_retry_semantics(current.body)
            )
        elif section_key == normalized("fail-closed boundaries"):
            errors.extend(f"{relative}: {error}" for error in validate_fail_closed(current.body))
        elif section_key == normalized("known gaps and unsupported behavior"):
            errors.extend(f"{relative}: {error}" for error in validate_known_gaps(current.body))
        elif section_key == normalized("source references"):
            errors.extend(f"{relative}: {error}" for error in validate_source_references(current.body))

    requirements = {
        section: tuple(anchors) for section, anchors in COMMON_SECTION_ANCHORS.items()
    }
    for section, anchors in CHAIN_SECTION_ANCHORS[filename].items():
        requirements[section] = requirements.get(section, ()) + tuple(anchors)

    for section, anchors in requirements.items():
        current = sections.get(normalized(section))
        if current is None:
            continue
        for anchor in anchors:
            if not bounded_anchor_present(current.body, anchor):
                errors.append(
                    f"{relative}: section '{section}' missing required anchor: {anchor}"
                )

    return errors


def main() -> int:
    errors: list[str] = []
    for filename in REQUIRED_GUIDES:
        errors.extend(check_guide(filename))

    for path in markdown_files_to_check():
        errors.extend(check_local_links(path, path.read_text(encoding="utf-8")))

    errors.extend(check_source_anchors())

    if not WORKFLOW.is_file():
        errors.append(f"missing workflow: {WORKFLOW.relative_to(ROOT)}")
    elif not workflow_has_validator_invocation(WORKFLOW.read_text(encoding="utf-8")):
        errors.append(".github/workflows/main.yml: missing active signing-doc validator invocation")

    if errors:
        for error in sorted(set(errors)):
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("verify_signing_docs: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
