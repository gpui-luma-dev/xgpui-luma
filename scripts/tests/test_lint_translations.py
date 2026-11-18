"""#265 regressions: each defect reported against the draft catalogs must fail the lint."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from lint_translations import problems  # noqa: E402


class TranslationLint(unittest.TestCase):
    def assertFlagged(self, english, translated, verbatim=False):
        self.assertTrue(problems(english, translated, verbatim), f'{english!r} -> {translated!r}')

    def assertClean(self, english, translated):
        self.assertEqual(problems(english, translated), [], f'{english!r} -> {translated!r}')

    def test_reported_defects_are_flagged(self):
        self.assertFlagged('…', '.' * 2867)
        self.assertFlagged('ms/Mpx', 'ms' + '/Mpx' * 68)
        self.assertFlagged('x', ' '.join(['X'] * 42))
        self.assertFlagged('3 x 4', '34 34 34 34 34 44 34 4')
        self.assertFlagged('Zoom +', 'تكبير')
        self.assertFlagged('Zoom −', 'تكبير')
        self.assertFlagged('−2–2 EV', '2  2 EV')
        self.assertFlagged('English', '中国語(簡体字)', verbatim=True)
        self.assertFlagged('English', 'العربية', verbatim=True)
        self.assertFlagged('Deutsch', '日本語 / English', verbatim=True)

    def test_mangled_samples_code_and_numbers_are_flagged(self):
        self.assertFlagged('שלום, world!', 'world、世界!')
        self.assertFlagged('compile pastel_waves.metal', 'pastel waves.metal をコンパイル')
        self.assertFlagged('zoom up to 10¹⁰×', '1010×までズーム')
        self.assertFlagged('Card { $a0 }\n3072 × 2048', 'Karte { $a0 } 3072 × 2048\n3072 × 2048')
        self.assertFlagged('60 Hz baseline · { $a0 }', 'خط الأساس 60 هرتز  { $a0 }')

    def test_locale_conventions_pass(self):
        self.assertClean('Loading…', 'Laden ...')
        self.assertClean('strength 0.6 · 10,000 cards', 'Stärke 0,6 · 10.000 Karten')
        self.assertClean('Compare two ways', '2つの方法を比較します')
        self.assertClean('Freeform · sixteen points', 'フリーフォーム・16ポイント')
        self.assertClean('−200–200 px', '−200〜200 px')
        self.assertClean('{ $a0 }%', '{ $a0 }٪')
        self.assertClean('Step ٣', 'Step ٣')
        self.assertClean('Zoom +', 'تكبير +')


if __name__ == '__main__':
    unittest.main()
