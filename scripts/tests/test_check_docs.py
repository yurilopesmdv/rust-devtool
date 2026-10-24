import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts.check_docs import check_root


class MarkdownLinksTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="rust-devtool-docs-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def test_relative_links_and_unicode_duplicate_headings(self):
        self.write("guide.md", "# Configuração\n## Repetido\n## Repetido\n")
        self.write(
            "nested/README.md",
            "[A](../guide.md#configuração)\n[B](../guide.md#repetido-1)",
        )
        self.assertEqual(check_root(self.root), (2, []))

    def test_missing_file_and_fragment_report_source_lines(self):
        self.write(
            "README.md", "# Início\n[quebrado](missing.md)\n[âncora](#ausente)\n"
        )
        _, errors = check_root(self.root)
        self.assertEqual(len(errors), 2)
        self.assertIn("README.md:2: arquivo/diretório ausente: missing.md", errors)
        self.assertIn("README.md:3: fragmento ausente: #ausente", errors)

    def test_spaces_encoded_paths_parentheses_and_titles(self):
        self.write("folder/My File (1).md", "# Uma seção\n")
        self.write(
            "README.md",
            '[A](<folder/My File (1).md#uma-seção>)\n[B](folder/My%20File%20(1).md#uma-seção "Título")',
        )
        self.assertEqual(check_root(self.root), (2, []))

    def test_references_images_html_anchor_and_setext_heading(self):
        self.write("guide.md", 'Título\n======\n<a id="custom-anchor"></a>\n')
        self.write("image.svg", "<svg />")
        self.write(
            "README.md",
            "[A][guide]\n[B](guide.md#custom-anchor)\n![image](image.svg)\n\n[guide]: guide.md#título\n",
        )
        self.assertEqual(check_root(self.root), (2, []))

    def test_external_urls_fences_comments_and_inline_code_are_ignored(self):
        self.write(
            "README.md",
            "[web](https://invalid.example/page)\n[mail](mailto:x@example.com)\n```md\n[bad](missing.md)\n```\n~~~\n[bad](missing.md)\n~~~\n`[bad](missing.md)`\n<!-- [bad](missing.md) -->\n",
        )
        self.assertEqual(check_root(self.root), (1, []))

    def test_excluded_directories_and_symlinks_are_not_scanned(self):
        self.write("README.md", "# OK\n")
        for name in ["node_modules", "target", ".venv"]:
            self.write(f"{name}/bad.md", "[bad](missing.md)")
        (self.root / "alias").symlink_to(self.root / "target", target_is_directory=True)
        self.assertEqual(check_root(self.root), (1, []))

    def test_root_paths_and_encoded_fragments(self):
        self.write("README.md", "[A](/guide.md#se%C3%A7%C3%A3o)\n")
        self.write("guide.md", "# Seção\n")
        self.assertEqual(check_root(self.root), (2, []))

    def test_links_inside_titles_are_not_parsed_again(self):
        self.write("guide.md", "# OK\n")
        self.write("README.md", '[A](guide.md "[not a link](missing.md)")')
        self.assertEqual(check_root(self.root), (2, []))

    def test_heading_inline_code_keeps_underscores(self):
        self.write("guide.md", "## Usar `foo_bar_baz`\n")
        self.write("README.md", "[A](guide.md#usar-foo_bar_baz)")
        self.assertEqual(check_root(self.root), (2, []))

    def test_destinations_cannot_escape_root(self):
        self.write("README.md", "[A](../outside.md)\n")
        self.assertIn("fora do repositório", check_root(self.root)[1][0])

    def test_cli_returns_nonzero_for_broken_link(self):
        self.write("README.md", "[bad](missing.md)")
        result = subprocess.run(
            [
                sys.executable,
                str(Path(__file__).resolve().parents[1] / "check_docs.py"),
                "--root",
                str(self.root),
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("README.md:1", result.stdout)


if __name__ == "__main__":
    unittest.main()
