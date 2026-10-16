#!/usr/bin/env python3
"""check_verdict_table.py — the corpus table and the corpus files must agree.

tests/checker-corpus/VERDICTS.md is the *human-normative record* of every
adversarial program's verdict; each `<NN>-*.hy` fixture carries the same
verdict on its machine-readable `// EXPECTED-VERDICT:` line.
tools/corpus_harness.py consumes the line. Nothing consumed the table, and
the two drifted:

  * two commits (2026-10-15, corpus 32→39 and 32→42) renumbered the FILES
    once 25-41 were taken, but the rows they added kept row number 32 — so
    the table held three rows numbered 32 and no row at all for 39 or 42;
  * a reader asking "what is program 39?" got nothing, and "program 32" got
    three different programs with no way to tell which was live.

Silent drift in the document that defines the spec is the same defect class
as a merged test nothing runs: the record stops describing reality while
still reading as authoritative. This guard fails closed on it.

For every numbered table row (`| NN | NN-name | VERDICT | ... |`) the row's
name cell must be the stem of an existing `<NN>-*.hy`; the file it names
must carry that row's number; and the row's verdict must match the fixture's
header. Every fixture must be named by exactly one row, and no row number
may repeat.

Usage: python3 tools/check_verdict_table.py [--table PATH] [--corpus DIR]
Exit:  0 = table and corpus agree; 1 = drift found; 2 = guard misuse.
"""

import argparse
import os
import re
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_TABLE = os.path.join(REPO_ROOT, "tests", "checker-corpus", "VERDICTS.md")
DEFAULT_CORPUS = os.path.join(REPO_ROOT, "tests", "checker-corpus")

# A numbered table row: `| 32 | 32-some-name | REJECT | A1c | ... |`.
ROW_RE = re.compile(r"^\|\s*(\d{2})\s*\|\s*([^|]*?)\s*\|")
# The machine-readable header inside a fixture.
HEADER_RE = re.compile(
    r"^//\s*EXPECTED-VERDICT:\s*(ACCEPT-MANAGED|ACCEPT|REJECT)\b\s*(\S*)", re.M
)
# `<NN>-rest.hy`
FILE_RE = re.compile(r"^(\d{2})-(.+)\.hy$")
# A markdown table separator, e.g. `|----|-----|` — never a row.
SEPARATOR_RE = re.compile(r"^\|[\s:|-]+\|?$")


def normalize_verdict(text):
    """ACCEPT-MANAGED / ACCEPT / REJECT, ignoring `(contested)` and spacing."""
    compact = re.sub(r"[^A-Z-]", "", text.upper())
    if "ACCEPT-MANAGED" in compact:
        return "ACCEPT-MANAGED"
    if "ACCEPT" in compact:
        return "ACCEPT"
    if "REJECT" in compact:
        return "REJECT"
    return None


def parse_table(table_text):
    """[(num, name, verdict, raw_verdict_cell)] for every numbered row."""
    rows = []
    for line in table_text.splitlines():
        if SEPARATOR_RE.match(line):
            continue
        match = ROW_RE.match(line)
        if not match:
            continue
        num, name = match.group(1), match.group(2)
        if not name or name.startswith("-"):
            continue
        cells = line.split("|")
        raw = cells[3].strip() if len(cells) > 3 else ""
        rows.append((num, name, normalize_verdict(raw), raw))
    return rows


def parse_corpus(corpus_dir):
    """({num: entry}, problems) for every `<NN>-*.hy` fixture."""
    found = {}
    problems = []
    for filename in sorted(os.listdir(corpus_dir)):
        match = FILE_RE.match(filename)
        if not match:
            continue
        num, stem = match.group(1), match.group(2)
        with open(os.path.join(corpus_dir, filename), encoding="utf-8") as handle:
            header = HEADER_RE.search(handle.read())
        if header is None:
            problems.append(f"{filename} — no `// EXPECTED-VERDICT:` line")
            continue
        found[num] = {
            "file": filename,
            "stem": f"{num}-{stem}",
            "verdict": normalize_verdict(header.group(1)),
            "rule": header.group(2),
        }
    return found, problems


def check(table_text, corpus_dir):
    """Every disagreement between the table and the corpus fixtures."""
    rows = parse_table(table_text)
    corpus, problems = parse_corpus(corpus_dir)

    counts = {}
    for num, _, _, _ in rows:
        counts[num] = counts.get(num, 0) + 1
    for num in sorted(counts):
        if counts[num] > 1:
            problems.append(
                f"row number {num} appears {counts[num]} times — the number is "
                "not a usable key"
            )

    numbers_by_stem = {entry["stem"]: num for num, entry in corpus.items()}
    # `<NN>-rest` -> NN, so a row that kept a dead number is reported as the
    # renumber it is rather than as an unknown name.
    numbers_by_suffix = {
        entry["stem"].split("-", 1)[1]: num for num, entry in corpus.items()
    }

    for num, name, verdict, raw in rows:
        if name in numbers_by_stem:
            file_num = numbers_by_stem[name]
        elif "-" in name and name.split("-", 1)[1] in numbers_by_suffix:
            file_num = numbers_by_suffix[name.split("-", 1)[1]]
            problems.append(
                f"row {num} names `{name}`, but that fixture is numbered "
                f"{file_num} — the row kept a stale number (renumber it to "
                f"{file_num})"
            )
            continue
        else:
            problems.append(
                f"row {num} names `{name}`, which is not a fixture in "
                f"{os.path.basename(corpus_dir)}/"
            )
            continue
        if file_num != num:
            problems.append(
                f"row {num} names `{name}`, but that fixture is numbered "
                f"{file_num} — the row kept a stale number"
            )
            continue
        entry = corpus[file_num]
        if verdict != entry["verdict"]:
            problems.append(
                f"{entry['file']} — table says {raw!r}, the header says "
                f"{entry['verdict']}"
            )

    named_stems = {name for _, name, _, _ in rows}
    for num, entry in sorted(corpus.items()):
        if entry["stem"] not in named_stems:
            problems.append(
                f"{entry['file']} — no VERDICTS.md row names this fixture"
            )

    return problems, rows, corpus


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--table", default=DEFAULT_TABLE,
                    help="the human-normative verdict table")
    ap.add_argument("--corpus", default=DEFAULT_CORPUS,
                    help="directory holding the `<NN>-*.hy` fixtures")
    args = ap.parse_args()

    if not os.path.isfile(args.table):
        print(f"VERDICT TABLE ERROR: table not found: {args.table}", file=sys.stderr)
        return 2
    if not os.path.isdir(args.corpus):
        print(f"VERDICT TABLE ERROR: corpus dir not found: {args.corpus}",
              file=sys.stderr)
        return 2

    with open(args.table, encoding="utf-8") as handle:
        table_text = handle.read()

    problems, rows, corpus = check(table_text, args.corpus)
    if not rows:
        print(f"VERDICT TABLE ERROR: no numbered rows parsed from {args.table}; "
              "the guard cannot verify anything", file=sys.stderr)
        return 2
    if not corpus:
        print(f"VERDICT TABLE ERROR: no `<NN>-*.hy` fixtures in {args.corpus}; "
              "the guard cannot verify anything", file=sys.stderr)
        return 2

    if problems:
        print("VERDICTS.md AND THE CORPUS DISAGREE — the normative record has "
              "stopped describing the fixtures:")
        for problem in problems:
            print(f"  {problem}")
        print("  Fix the TABLE (renumber the row to the file's number) or the "
              "FIXTURE, then re-run. Never silence this by deleting a check.")
        return 1

    print(f"verdict table: OK ({len(corpus)} fixture(s), {len(rows)} table row(s))")
    return 0


if __name__ == "__main__":
    sys.exit(main())
