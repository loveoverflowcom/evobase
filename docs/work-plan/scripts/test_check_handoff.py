"""Regression tests for the planning-only document validator."""

import tempfile
import unittest
from pathlib import Path

from check_handoff import SKILLS, allowed_path, validate


class HandoffChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in SKILLS:
            path = self.root / ".agents" / "skills" / name / "SKILL.md"
            path.parent.mkdir(parents=True)
            path.write_text(f"---\nname: {name}\ndescription: Fixture guidance\n---\n# Skill\n")
        self.doc = self.root / "docs" / "work-plan" / "README.md"
        self.doc.parent.mkdir(parents=True)
        self.doc.write_text("# Handoff\n")

    def check(self, changed=None):
        return validate(self.root, [] if changed is None else changed)["errors"]

    def test_valid_fixture(self):
        self.assertEqual([], self.check(["AGENTS.md", "docs/work-plan/README.md"]))

    def test_missing_relative_link(self):
        self.doc.write_text("# Handoff\n[missing](missing.md)\n")
        self.assertTrue(any(x.startswith("missing link:") for x in self.check()))

    def test_external_link_is_not_local_file(self):
        self.doc.write_text("# Handoff\n[Vision](https://github.com/example/repo/issues/1)\n")
        self.assertEqual([], self.check())

    def test_missing_heading_anchor(self):
        self.doc.write_text("# Handoff\n[bad](#missing)\n")
        self.assertTrue(any(x.startswith("missing anchor:") for x in self.check()))

    def test_product_change_outside_planning_scope(self):
        self.assertIn("outside planning scope: apps/web/src/lib.rs", self.check(["apps/web/src/lib.rs"]))
        self.assertFalse(allowed_path("Cargo.toml"))

    def test_missing_required_skill(self):
        (self.root / ".agents/skills/evobase-engineering/SKILL.md").unlink()
        self.assertIn("missing skill: evobase-engineering", self.check())

    def test_no_absolute_path_or_outside_link(self):
        self.doc.write_text("# Handoff\n[outside](../../../../outside.md)\n")
        self.assertTrue(any(x.startswith("link outside repository:") for x in self.check()))


if __name__ == "__main__":
    unittest.main()
