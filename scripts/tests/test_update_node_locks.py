import copy
import json
import unittest

from ferramentas.boilerplate.scripts.update_node_locks import normalize


class NodeLockMaintenanceTest(unittest.TestCase):
    def test_base_normalizes_only_root_identity_and_engine(self):
        lock = {
            "name": "fixture",
            "packages": {
                "": {"name": "fixture", "engines": {"node": ">=24 <25.0.0"}},
                "node_modules/fixture": {
                    "version": "1.0.0",
                    "integrity": "sha512-test",
                },
            },
        }
        original = copy.deepcopy(lock)
        result = json.loads(normalize(lock, True))
        self.assertEqual(result["name"], "{{PROJECT_NAME}}")
        self.assertEqual(result["packages"][""]["engines"]["node"], "{{NODE_ENGINE}}")
        self.assertEqual(
            result["packages"]["node_modules/fixture"],
            original["packages"]["node_modules/fixture"],
        )

    def test_superset_preserves_concrete_identity(self):
        lock = {"name": "express-ts", "packages": {"": {"name": "express-ts"}}}
        self.assertEqual(json.loads(normalize(lock, False)), lock)
