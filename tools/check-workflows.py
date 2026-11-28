#!/usr/bin/env python3
"""Parse every tracked .github/workflows/*.yml|*.yaml and fail on the first kind of error that makes GitHub record "workflow file issue" runs
instead of running the jobs (found 2026-11-28: selfhosted.yml had an unquoted ": " in a step name, invalid since #35, and no job ran for
a day; nothing in CI said so). Checks: valid YAML; top level has `on` and a `jobs` mapping; every job has `runs-on` or `uses` (and `steps`
for `runs-on` jobs). This is NOT actionlint (expressions, action inputs and permissions are not checked).
Enumerates with `git ls-files` (never a disk walk, so other lanes' worktrees are not read). Usage:
    python tools/check-workflows.py            # check the repository
    python tools/check-workflows.py --control  # the checker's own controls: tools/fixtures/broken-workflow.yml.txt must be REJECTED and
                                               # good-workflow.yml.txt ACCEPTED, else exit 1
Needs PyYAML (`pip install pyyaml`)."""
import subprocess
import sys

try:
    import yaml
except ImportError:
    sys.exit("check-workflows: PyYAML is not installed (pip install pyyaml)")


def problems(path):
    try:
        with open(path, encoding="utf8") as f:
            d = yaml.safe_load(f)
    except yaml.YAMLError as e:
        return [f"invalid YAML: {str(e).splitlines()[0]} ({str(e).splitlines()[-1].strip()})"]
    if not isinstance(d, dict):
        return ["top level is not a mapping"]
    out = []
    if "on" not in d and True not in d:  # PyYAML reads the bare key `on` as boolean True
        out.append("no `on:` trigger")
    jobs = d.get("jobs")
    if not isinstance(jobs, dict) or not jobs:
        out.append("no `jobs:` mapping")
        return out
    for name, job in jobs.items():
        if not isinstance(job, dict):
            out.append(f"job `{name}` is not a mapping")
        elif "uses" not in job and "runs-on" not in job:
            out.append(f"job `{name}` has neither `runs-on` nor `uses`")
        elif "uses" not in job and not isinstance(job.get("steps"), list):
            out.append(f"job `{name}` has no `steps` list")
    return out


def control():
    bad = problems("tools/fixtures/broken-workflow.yml.txt")
    good = problems("tools/fixtures/good-workflow.yml.txt")
    ok = True
    if not bad:
        print("CONTROL FAIL: the broken fixture was ACCEPTED (the checker cannot see the defect it exists for)")
        ok = False
    else:
        print("control ok: broken fixture rejected:", bad[0])
    if good:
        print("CONTROL FAIL: the good fixture was REJECTED:", good)
        ok = False
    else:
        print("control ok: good fixture accepted")
    return ok


def main():
    if "--control" in sys.argv:
        sys.exit(0 if control() else 1)
    files = subprocess.run(
        ["git", "ls-files", ".github/workflows/*.yml", ".github/workflows/*.yaml"],
        capture_output=True, text=True, check=True,
    ).stdout.split()
    print(f"check-workflows: {len(files)} workflow file(s) from `git ls-files`")
    if not files:
        sys.exit("check-workflows: no workflow files found (a checker that checks nothing passes nothing)")
    if not control():
        sys.exit(1)
    bad = 0
    for p in files:
        pr = problems(p)
        print(("FAIL " if pr else "ok   ") + p)
        for x in pr:
            print("     ", x)
        bad += bool(pr)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
