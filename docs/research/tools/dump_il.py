"""Print decoded CIL anchors for one method from the REA evidence export.

Used to re-verify citations in docs/research/BEHAVIOR_SPEC.md, e.g.:

    python3 -I docs/research/tools/dump_il.py ev_566abdd6 0x06000101

Reads the export from ../camera-mouse-reverse/research/rea-evidence.json (override with
REA_EVIDENCE=<path>). Output is printed only; nothing decompiled is stored in this repo.
Stdlib only.
"""

import json
import os
import sys
from pathlib import Path

DEFAULT = Path(__file__).resolve().parents[3].parent / "camera-mouse-reverse/research/rea-evidence.json"


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    prefix, token = sys.argv[1], sys.argv[2].lower()
    path = Path(os.environ.get("REA_EVIDENCE", DEFAULT))
    data = json.loads(path.read_text())

    names: dict[str, str] = {}
    for rec in data["records"]:
        nr = rec["normalized_result"]
        if not isinstance(nr, dict):
            continue
        for m in nr.get("member_refs") or []:
            names[m["token"]] = f'{m["parent_name"]}::{m["name"]}'
        for m in nr.get("methods") or []:
            names[m["token"]] = f'{m["declaring_type"]}.{m["name"]}'

    matches = [r for r in data["records"] if r["evidence_id"].startswith(prefix)]
    if len(matches) != 1:
        print(f"{len(matches)} records match prefix {prefix!r}", file=sys.stderr)
        return 1
    methods = matches[0]["normalized_result"].get("methods") or []
    method = next((m for m in methods if m["token"] == token), None)
    if method is None:
        print(f"token {token} not in {matches[0]['evidence_id']}", file=sys.stderr)
        return 1

    sig = method["signature"]
    print(f'{matches[0]["evidence_id"]}\n{token} {method["declaring_type"]}.{method["name"]}'
          f'({", ".join(sig["parameter_types"])}) -> {sig["return_type"]}  rva={hex(method["rva"])}')
    for a in method["body"]["anchors"]:
        operand = a["operand"] or ""
        if a["operand_kind"] == "method":
            operand = f"{names.get(operand, operand)} [{operand}]"
        print(f'  IL_{a["il_offset"]:04d}  {a["opcode"]:<12} {operand}')
    return 0


if __name__ == "__main__":
    sys.exit(main())
