"""External frontend acceptance tests; every compiler invocation uses a prebuilt binary."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from producer import counter, blueprint

ROOT = Path(__file__).resolve().parents[2]
BINARY = Path(os.environ["SAND_PROGRAM_BIN"]).resolve()
FIXTURES = ROOT / "tests" / "fixtures" / "program"
GOLDEN = {p.relative_to(FIXTURES / "golden").as_posix(): p.read_bytes()
          for p in (FIXTURES / "golden").rglob("*") if p.is_file()}

class Programs(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.cwd = Path(self.temp.name).resolve()
        self.env = {**os.environ, "PATH": str(self.cwd / "no-cargo")}

    def run_sand(self, args, document=None, success=True):
        body = json.dumps(document) if isinstance(document, dict) else document
        process = subprocess.run([str(BINARY), "program", *args], input=body, text=True,
                                 capture_output=True, cwd=self.cwd, env=self.env, timeout=30)
        response = json.loads(process.stdout)
        self.assertEqual(response["success"], success, response)
        self.assertEqual(process.returncode == 0, success, process.stderr)
        self.assertEqual(response["schema_version"], 1)
        return response

    def compile(self, document, name="pack"):
        output = self.cwd / name
        response = self.run_sand(["compile", "--input", "-", "--output", str(output)], document)
        self.assertNotIn("file_contents", response)
        files = {p.relative_to(output).as_posix(): p.read_bytes() for p in output.rglob("*")
                 if p.is_file() and p.name != ".sand-build-manifest.json"}
        return files

    def test_python_json_blueprint_and_split_match_golden(self):
        schema = self.run_sand(["schema"])["schema"]
        program = counter()
        Draft202012Validator(schema).validate(program)
        self.assertEqual(self.compile(program), GOLDEN)
        self.assertEqual(self.compile(json.loads((FIXTURES / "counter.sand.json").read_text())), GOLDEN)
        graph = {"nodes": [{"id": "t", "kind": "tick"}, {"id": "p", "kind": "players"}, {"id": "i", "kind": "increment"}],
                 "edges": [{"from": "t", "to": "p"}, {"from": "p", "to": "i"}]}
        self.assertEqual(self.compile(blueprint(graph)), GOLDEN)
        for node in graph["nodes"]:
            node["layout"] = [999, -12]
        self.assertEqual(self.compile(blueprint(graph)), GOLDEN)
        response = self.run_sand(["compile", "--input", str(FIXTURES / "split/program.sand.json"),
                                  "--output", str(self.cwd / "split"), "--include-files"])
        self.assertEqual({key: value.encode() for key, value in response["file_contents"].items()}, GOLDEN)

    def test_domain_schemas_resolve_entirely_offline(self):
        root = ROOT / "schemas/program"
        registry = Registry().with_resources(
            (path.resolve().as_uri(), Resource.from_contents(json.loads(path.read_text())))
            for path in root.glob("*.json"))
        path = root / "program.schema.json"
        schema = {**json.loads(path.read_text()), "$id": path.resolve().as_uri()}
        Draft202012Validator(schema, registry=registry).validate(counter())
        schema = json.loads((root / "module.schema.json").read_text())
        Draft202012Validator(schema).validate(counter()["modules"][0])

    def test_existing_rust_frontend_matches_complete_golden(self):
        pack = Path(os.environ["SAND_RUST_PACK"])
        output = {p.relative_to(pack).as_posix(): p.read_bytes() for p in pack.rglob("*")
                  if p.is_file() and p.name != ".sand-build-manifest.json"}
        self.assertEqual(output, GOLDEN)

    def test_blueprint_rejects_ambiguity_and_cycles(self):
        graph = {"nodes": [{"id": "t", "kind": "tick"}, {"id": "p", "kind": "players"}, {"id": "i", "kind": "increment"}],
                 "edges": [{"from": "t", "to": "p"}, {"from": "p", "to": "i"}]}
        for edge in [{"from": "t", "to": "i"}, {"from": "i", "to": "t"}]:
            invalid = copy.deepcopy(graph); invalid["edges"].append(edge)
            with self.assertRaises(ValueError): blueprint(invalid)

    def test_invalid_mutations_preserve_existing_output(self):
        self.compile(counter())
        output = self.cwd / "pack"
        before = {str(p): p.read_bytes() for p in output.rglob("*") if p.is_file()}
        mutations = [
            lambda p: p.update(format_version=2),
            lambda p: p.update(requires=["events"]),
            lambda p: p["modules"][0]["states"][0].update(revision=2),
            lambda p: p["modules"][0]["functions"][1].update(context="server"),
            lambda p: p["modules"][0]["functions"][1]["body"][0]["action"].update(value=True),
            lambda p: p["modules"][0]["functions"][1]["body"][0]["action"].update(op="teleport"),
            lambda p: p["modules"].append(copy.deepcopy(p["modules"][0])),
            lambda p: p["modules"][0]["functions"][1]["body"][0]["action"]["score"].update(field="missing"),
        ]
        for mutate in mutations:
            program = counter(); mutate(program)
            self.run_sand(["compile", "--input", "-", "--output", str(output)], program, success=False)
            self.assertEqual({str(p): p.read_bytes() for p in output.rglob("*") if p.is_file()}, before)

    def test_manifest_and_filesystem_conflicts(self):
        for modules in [["../escape"], ["/absolute"], ["module", "module"], ["missing"]]:
            program = counter(); program["modules"] = modules
            path = self.cwd / "program.json"; path.write_text(json.dumps(program))
            self.run_sand(["check", "--input", str(path)], success=False)
        program = counter(); program["modules"] = ["module.json"]
        self.run_sand(["check", "--input", "-"], program, success=False)
        target = self.cwd / "module.json"; target.symlink_to(FIXTURES / "split/state.sand.json")
        path = self.cwd / "program.json"; path.write_text(json.dumps(program))
        self.run_sand(["check", "--input", str(path)], success=False)
        self.compile(counter())
        owned = self.cwd / "pack/data/demo/function/increment.mcfunction"; owned.write_text("edited")
        self.run_sand(["compile", "--input", "-", "--output", str(self.cwd / "pack")], counter(), success=False)
        self.assertEqual(owned.read_text(), "edited")

if __name__ == "__main__":
    unittest.main()
