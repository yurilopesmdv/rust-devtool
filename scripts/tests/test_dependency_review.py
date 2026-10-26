import unittest

from scripts.dependency_review import review


class DependencyReviewTest(unittest.TestCase):
    def test_template_updates_do_not_masquerade_as_workspace_updates(self):
        result = review(
            [
                "ferramentas/boilerplate/templates/express-ts/package.json",
                "ferramentas/boilerplate/templates/express-ts/package-lock.json",
            ]
        )
        self.assertEqual(result["scopes"], ["npm-templates"])
        self.assertTrue(
            any("commonjs" in command for command in result["checks"]["npm-templates"])
        )

    def test_coordinated_changes_report_all_affected_ecosystems(self):
        result = review(
            [
                "Cargo.lock",
                ".github/workflows/quality.yml",
                "ferramentas/boilerplate/integrations/python-locks/postgres-1-1.lock",
                "ferramentas/boilerplate/templates/rust-axum/Cargo.toml",
            ]
        )
        self.assertEqual(
            result["scopes"],
            ["actions", "python-templates", "rust-templates", "workspace"],
        )
