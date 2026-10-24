#!/usr/bin/env python3
"""Check local Markdown links without network access or external packages."""

import argparse
import html
import os
import re
from pathlib import Path
from urllib.parse import unquote, urlsplit

EXCLUDED = {
    ".git",
    ".agents",
    ".codex",
    ".aws",
    "target",
    "node_modules",
    ".venv",
    "__pycache__",
    ".pytest_cache",
    ".ruff_cache",
    ".next",
    "dist",
    "build",
}


def prose(source):
    """Keep line numbers while excluding comments and fenced code."""
    source = re.sub(
        r"<!--.*?-->",
        lambda match: "\n" * match.group().count("\n"),
        source,
        flags=re.DOTALL,
    )
    fence = None
    for line in source.splitlines():
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if marker:
            value = marker.group(1)
            if fence is None:
                fence = value
            elif value[0] == fence[0] and len(value) >= len(fence):
                fence = None
            yield ""
        elif fence:
            yield ""
        else:
            yield line


def slug(title):
    title = re.sub(r"<[^>]+>", "", title)
    title = re.sub(r"!?\[([^]]+)\]\([^)]*\)", r"\1", title)
    title = title.replace("`", "")
    title = re.sub(r"(?<!\w)([*_~]+)(.+?)\1(?!\w)", r"\2", title)
    title = html.unescape(title).strip().lower()
    return re.sub(r"\s", "-", re.sub(r"[^\w\-\s]", "", title))


def anchors(source):
    result = set()
    headings = set()
    previous = ""
    for line in prose(source):
        for match in re.finditer(r"\b(?:id|name)\s*=\s*(['\"])(.*?)\1", line):
            result.add(html.unescape(match.group(2)))
        heading = re.match(r"^ {0,3}#{1,6}(?:\s+|$)(.*)", line)
        title = None
        if heading:
            title = re.sub(r"\s+#+\s*$", "", heading.group(1))
        elif previous.strip() and re.fullmatch(r" {0,3}(?:=+|-+)\s*", line):
            title = previous
        if title is not None:
            base = slug(title)
            candidate = base
            suffix = 0
            while candidate in headings:
                suffix += 1
                candidate = f"{base}-{suffix}"
            headings.add(candidate)
            result.add(candidate)
        previous = line
    return result


def destination(text, start):
    """Read an angle destination or a bare destination with balanced parentheses."""
    cursor = start
    while cursor < len(text) and text[cursor].isspace():
        cursor += 1
    if cursor == len(text):
        return None, cursor
    if text[cursor] == "<":
        end = cursor + 1
        while end < len(text):
            if text[end] == "\\":
                end += 2
                continue
            if text[end] == ">":
                return text[cursor + 1 : end], end + 1
            end += 1
        return None, cursor
    begin = cursor
    depth = 0
    while cursor < len(text):
        char = text[cursor]
        if char == "\\":
            cursor += 2
            continue
        if char.isspace() or (char == ")" and depth == 0):
            break
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        cursor += 1
    if depth:
        return None, cursor
    return text[begin:cursor], cursor


def closing_link(text, cursor):
    while cursor < len(text) and text[cursor].isspace():
        cursor += 1
    if cursor < len(text) and text[cursor] in "\"'":
        quote = text[cursor]
        cursor += 1
        while cursor < len(text) and text[cursor] != quote:
            cursor += 2 if text[cursor] == "\\" else 1
        cursor += 1
        while cursor < len(text) and text[cursor].isspace():
            cursor += 1
    if cursor < len(text) and text[cursor] == ")":
        return cursor + 1
    return None


def closing_bracket(text, start):
    depth = 1
    cursor = start + 1
    while cursor < len(text):
        if text[cursor] == "\\":
            cursor += 2
            continue
        if text[cursor] == "[":
            depth += 1
        elif text[cursor] == "]":
            depth -= 1
            if not depth:
                return cursor
        cursor += 1
    return None


def reference_id(label):
    return " ".join(label.split()).lower()


def links(source):
    lines = list(prose(source))
    references = {}
    definitions = set()
    for number, line in enumerate(lines):
        match = re.match(r"^ {0,3}\[([^]]+)\]:\s*", line)
        if match:
            target, _ = destination(line, match.end())
            if target is not None:
                references.setdefault(reference_id(match.group(1)), target)
                definitions.add(number)
    for number, original in enumerate(lines):
        if number in definitions:
            continue
        line = re.sub(r"(`+).*?\1", "", original)
        cursor = 0
        while cursor < len(line):
            if line[cursor] == "\\":
                cursor += 2
                continue
            if line[cursor] != "[":
                cursor += 1
                continue
            end = closing_bracket(line, cursor)
            if end is None:
                break
            label = line[cursor + 1 : end]
            after = end + 1
            target = None
            if after < len(line) and line[after] == "(":
                target, after = destination(line, after + 1)
                link_end = closing_link(line, after)
                if link_end is None:
                    target = None
                else:
                    after = link_end
            elif after < len(line) and line[after] == "[":
                ref_end = closing_bracket(line, after)
                if ref_end is not None:
                    target = references.get(
                        reference_id(line[after + 1 : ref_end] or label)
                    )
                    after = ref_end + 1
            else:
                target = references.get(reference_id(label))
            if target is not None:
                yield number + 1, re.sub(r"\\([\\()\[\]<> ])", r"\1", target)
            cursor = max(after, end + 1)


def markdown_files(root):
    for parent, directories, files in os.walk(root, followlinks=False):
        directories[:] = sorted(
            name
            for name in directories
            if name not in EXCLUDED and not (Path(parent) / name).is_symlink()
        )
        for name in sorted(files):
            path = Path(parent) / name
            if path.suffix.lower() == ".md" and not path.is_symlink():
                yield path


def check_root(root):
    root = Path(root).resolve()
    documents = list(markdown_files(root))
    errors = []
    cached_anchors = {}
    for document in documents:
        source = document.read_text(encoding="utf-8")
        for line, target in links(source):
            location = f"{document.relative_to(root)}:{line}"
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith(
                "//"
            ):
                continue
            try:
                url = urlsplit(html.unescape(target))
                path = unquote(url.path)
                resolved = (
                    root / path.lstrip("/")
                    if path.startswith("/")
                    else document.parent / path
                    if path
                    else document
                ).resolve()
                resolved.relative_to(root)
            except ValueError:
                errors.append(
                    f"{location}: destino fora do repositório ou inválido: {target}"
                )
                continue
            if not resolved.exists():
                errors.append(f"{location}: arquivo/diretório ausente: {target}")
                continue
            if url.fragment and resolved.is_file() and resolved.suffix.lower() == ".md":
                if resolved not in cached_anchors:
                    cached_anchors[resolved] = anchors(
                        resolved.read_text(encoding="utf-8")
                    )
                if unquote(url.fragment) not in cached_anchors[resolved]:
                    errors.append(f"{location}: fragmento ausente: {target}")
    return len(documents), errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    args = parser.parse_args()
    if not args.root.is_dir():
        parser.error("--root precisa apontar para um diretório existente")
    count, errors = check_root(args.root)
    for error in errors:
        print(error)
    print(f"{count} documentos Markdown; {len(errors)} links locais inválidos.")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
