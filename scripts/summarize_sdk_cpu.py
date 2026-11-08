#!/usr/bin/env python3
"""Summarize xctrace time-profile XML for the SDK fixture process only."""
import argparse
from collections import Counter
import json
from pathlib import Path
import xml.etree.ElementTree as ET


def summarize(path, start_ns=2_000_000_000, end_ns=12_000_000_000):
    if end_ns <= start_ns:
        raise ValueError("invalid sample window")
    root = ET.parse(path).getroot()
    definitions = {element.attrib["id"]: element for element in root.iter() if "id" in element.attrib}

    def resolve(element):
        if element is None:
            return None
        return definitions.get(element.get("ref"), element)

    def text(element):
        item = resolve(element)
        return item.text if item is not None else None

    main_weight = all_weight = samples = observed_main_rows = 0
    leaf = Counter()
    sdk = Counter()
    for row in root.iter("row"):
        process = resolve(row.find("process"))
        if process is None or not process.get("fmt", "").startswith("luma-sdk-perf ("):
            continue
        if text(row.find("thread-state")) != "Running":
            continue
        thread = resolve(row.find("thread"))
        is_main = thread is not None and thread.get("fmt", "").startswith("Main Thread")
        observed_main_rows += int(is_main)
        timestamp = text(row.find("sample-time"))
        if timestamp is None or not start_ns <= int(timestamp) < end_ns:
            continue
        weight = int(text(row.find("weight")) or 0)
        all_weight += weight
        if not is_main:
            continue
        main_weight += weight
        samples += 1
        stack = resolve(row.find("tagged-backtrace"))
        names = [] if stack is None else [resolve(frame).get("name", "unknown") for frame in stack.findall("frame")]
        if names:
            leaf[names[0]] += weight
        for name in set(names):
            if "xgpui_luma" in name:
                sdk[name] += weight
    if not observed_main_rows:
        raise ValueError("trace has no fixture main-thread samples")

    def top(counter):
        return [{"symbol": name, "sample_weight_ns": weight, "percent_of_main_weight": round(100 * weight / main_weight, 2)}
                for name, weight in counter.most_common(15)]

    return {"window_ns": [start_ns, end_ns], "main_samples": samples,
            "main_sample_weight_ns": main_weight, "all_thread_sample_weight_ns": all_weight,
            "estimated_main_cpu_percent": round(100 * main_weight / (end_ns - start_ns), 2),
            "leaf_symbols": top(leaf), "inclusive_sdk_symbols": top(sdk)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = json.dumps(summarize(args.input), indent=2) + "\n"
    if args.output:
        args.output.write_text(result)
    else:
        print(result)


if __name__ == "__main__":
    main()
