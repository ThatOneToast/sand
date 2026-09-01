import importlib.util
import tempfile
import unittest
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1] / "validate_worldbuild_profiles.py"
SPEC = importlib.util.spec_from_file_location("validate_worldbuild_profiles", MODULE)
profiles = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(profiles)


class WorldbuildProfileTests(unittest.TestCase):
    def test_parse_properties_ignores_comments_and_splits_once(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "server.properties"
            path.write_text("# comment\nview-distance=6\ncustom=a=b\n", encoding="utf-8")
            self.assertEqual(
                profiles.parse_properties(path),
                {"view-distance": "6", "custom": "a=b"},
            )


if __name__ == "__main__":
    unittest.main()
