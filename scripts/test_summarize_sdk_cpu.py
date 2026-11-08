import tempfile
import unittest
from pathlib import Path
from summarize_sdk_cpu import summarize


class CpuSummaryTests(unittest.TestCase):
    def run_summary(self, data):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "trace.xml"
            path.write_text(data)
            return summarize(path)

    def test_resolves_references_filters_startup_and_other_processes(self):
        row = '''<row><sample-time>{time}</sample-time><thread ref="t"/><process ref="{process}"/>
        <thread-state ref="s"/><weight ref="w"/><tagged-backtrace ref="b"/></row>'''
        xml = '''<trace><definitions><thread id="t" fmt="Main Thread (fixture)"/>
        <process id="p" fmt="luma-sdk-perf (123)"/><process id="other" fmt="other (456)"/>
        <thread-state id="s">Running</thread-state><weight id="w">1000000</weight>
        <tagged-backtrace id="b"><frame id="f" name="allocator"/><frame name="xgpui_luma::render"/></tagged-backtrace></definitions>'''
        xml += row.format(time=1_000_000_000, process="p")
        xml += row.format(time=3_000_000_000, process="p")
        xml += row.format(time=4_000_000_000, process="other") + "</trace>"
        result = self.run_summary(xml)
        self.assertEqual(result["main_samples"], 1)
        self.assertEqual(result["main_sample_weight_ns"], 1_000_000)
        self.assertEqual(result["leaf_symbols"][0]["symbol"], "allocator")
        self.assertEqual(result["inclusive_sdk_symbols"][0]["percent_of_main_weight"], 100)

    def test_missing_samples_are_an_error(self):
        with self.assertRaises(ValueError):
            self.run_summary("<trace/>")

    def test_idle_window_can_have_zero_observed_samples(self):
        result = self.run_summary('''<trace><row><sample-time>1000000000</sample-time>
        <thread fmt="Main Thread"/><process fmt="luma-sdk-perf (123)"/>
        <thread-state>Running</thread-state><weight>1000000</weight>
        </row></trace>''')
        self.assertEqual(result["main_samples"], 0)
        self.assertEqual(result["estimated_main_cpu_percent"], 0)
        self.assertEqual(result["leaf_symbols"], [])
