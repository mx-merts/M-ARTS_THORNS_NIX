#!/usr/bin/env python3
"""thorns-aide - AIDE wrapper for the THORNS system (file integrity monitoring).

Two independent baselines:
  boot     /boot. Legitimately changes on every rebuild, so it is checked
           BEFORE a rebuild (to catch out-of-band changes) and re-baselined
           AFTER a successful rebuild.
  persist  persistence points (ssh keys, autostart, user units, shell rc).
           Changes only when you accept them: sudo thorns-aide accept persist

Commands (run as root):
  init [boot|persist]       create baselines (first run: on a known-good system)
  check [boot|persist|all]  compare and report; --alert sends SPINE events
  pre-rebuild               check boot before a rebuild (exit 1 if it differs)
  post-rebuild              re-baseline boot after a successful rebuild
  accept <boot|persist>     accept the current state as the new baseline
  report <boot|persist>     show the last report
  status                    baselines and last results
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time

CONF_DIR = os.environ.get("THORNS_AIDE_CONF_DIR", "")
DB_DIR = os.environ.get("THORNS_AIDE_DB_DIR", "/var/lib/aide")
AIDE = os.environ.get("THORNS_AIDE_BIN", "aide")
NAMES = ["boot", "persist"]
LABEL = {"boot": "boot partition", "persist": "persistence files"}
SEVERITY = {"boot": "crit", "persist": "warn"}
STATUS_FILE = os.path.join(DB_DIR, "status.json")


def conf(name):
    return os.path.join(CONF_DIR, name + ".conf")


def db(name):
    return os.path.join(DB_DIR, name + ".db")


def report_path(name):
    return os.path.join(DB_DIR, name + ".report.txt")


def run_aide(name, mode):
    p = subprocess.run([AIDE, "-c", conf(name), "--" + mode],
                       capture_output=True, text=True)
    return p.returncode, p.stdout + p.stderr


def write_private(path, text):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        f.write(text)


def record(name, state, counts=None):
    try:
        with open(STATUS_FILE, encoding="utf-8") as f:
            st = json.load(f)
    except (OSError, ValueError):
        st = {}
    st[name] = {"time": int(time.time()), "state": state, "counts": counts or {}}
    write_private(STATUS_FILE, json.dumps(st))


def spine_event(sev, title, detail):
    exe = shutil.which("spine") or "/run/current-system/sw/bin/spine"
    try:
        subprocess.run([exe, "event", sev, "aide", title, detail],
                       check=False, timeout=30)
    except (OSError, subprocess.SubprocessError):
        print("note: could not reach spine; event not recorded", file=sys.stderr)


def parse_report(out):
    """Counts and changed paths from an AIDE report (summary part only)."""
    head = out.split("Detailed information about changes", 1)[0]
    counts = {}
    for key in ("Added", "Removed", "Changed"):
        m = re.search(r"%s entries:\s+(\d+)" % key, head)
        counts[key.lower()] = int(m.group(1)) if m else 0
    paths = re.findall(r"^[A-Za-z][^:\n]*: (/.*)$", head, re.M)
    return counts, paths


def init_db(name):
    rc, out = run_aide(name, "init")
    if rc != 0:
        print("aide --init failed for %s (rc=%d):\n%s" % (name, rc, out[-800:]),
              file=sys.stderr)
        return False
    os.replace(db(name) + ".new", db(name))
    os.chmod(db(name), 0o600)
    record(name, "clean")
    return True


def check_db(name, alert):
    """Returns 'clean', 'diff', 'error' or 'no-baseline'."""
    if not os.path.exists(db(name)):
        print("[%s] no baseline yet. Create it on a known-good system: "
              "sudo thorns-aide init" % name)
        record(name, "no-baseline")
        if alert:
            spine_event("warn", "AIDE has no %s baseline" % name,
                        "run: sudo thorns-aide init")
        return "no-baseline"

    rc, out = run_aide(name, "check")
    write_private(report_path(name), out)
    if rc == 0:
        print("[%s] clean: matches the baseline" % name)
        record(name, "clean")
        return "clean"
    if 1 <= rc <= 7:
        counts, paths = parse_report(out)
        summary = "%d added, %d removed, %d changed" % (
            counts["added"], counts["removed"], counts["changed"])
        print("[%s] DIFFERS from baseline: %s" % (name, summary))
        for p in paths[:15]:
            print("    " + p)
        if len(paths) > 15:
            print("    ... and %d more (sudo thorns-aide report %s)" % (len(paths) - 15, name))
        record(name, "diff", counts)
        if alert:
            shown = ", ".join(paths[:4]) + (" ..." if len(paths) > 4 else "")
            spine_event(SEVERITY[name], "AIDE: %s changed since baseline" % LABEL[name],
                        "%s: %s | review: sudo thorns-aide report %s | accept: "
                        "sudo thorns-aide accept %s" % (summary, shown, name, name))
        return "diff"
    print("[%s] aide error (rc=%d):\n%s" % (name, rc, out[-800:]), file=sys.stderr)
    record(name, "error")
    if alert:
        spine_event("warn", "AIDE check failed (%s)" % name, "exit code %d" % rc)
    return "error"


def pick(arg):
    return NAMES if arg in (None, "all") else [arg]


def cmd_init(args):
    ok = True
    for name in pick(args.name):
        if os.path.exists(db(name)) and not args.force:
            print("[%s] baseline exists; use --force to recreate it" % name)
            continue
        if init_db(name):
            print("[%s] baseline created" % name)
        else:
            ok = False
    return 0 if ok else 2


def cmd_check(args):
    states = [check_db(n, args.alert) for n in pick(args.name)]
    if "error" in states:
        return 2
    return 1 if "diff" in states else 0


def cmd_pre_rebuild(args):
    state = check_db("boot", args.alert)
    return 1 if state == "diff" else 0


def cmd_post_rebuild(args):
    if init_db("boot"):
        print("[boot] baseline refreshed after rebuild")
        return 0
    return 2


def cmd_accept(args):
    if not os.path.exists(db(args.name)):
        print("[%s] no baseline yet; use: sudo thorns-aide init %s" % (args.name, args.name))
        return 2
    if init_db(args.name):
        print("[%s] baseline updated to the current state" % args.name)
        return 0
    return 2


def cmd_report(args):
    try:
        with open(report_path(args.name), encoding="utf-8") as f:
            text = f.read()
    except OSError:
        print("no report yet for %s" % args.name)
        return 1
    print(text.split("The attributes of the (uncompressed) database", 1)[0].rstrip())
    return 0


def cmd_status(_args):
    try:
        with open(STATUS_FILE, encoding="utf-8") as f:
            st = json.load(f)
    except (OSError, ValueError):
        st = {}
    for name in NAMES:
        have = "baseline " + time.strftime("%Y-%m-%d %H:%M", time.localtime(os.path.getmtime(db(name)))) \
            if os.path.exists(db(name)) else "NO BASELINE"
        last = st.get(name)
        res = "never checked" if not last else "%s (%s)" % (
            last["state"], time.strftime("%Y-%m-%d %H:%M", time.localtime(last["time"])))
        print("%-8s: %s; last result: %s" % (name, have, res))
    return 0


def main():
    p = argparse.ArgumentParser(prog="thorns-aide", description="AIDE wrapper")
    sub = p.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("init")
    a.add_argument("name", nargs="?", choices=NAMES + ["all"])
    a.add_argument("--force", action="store_true")
    a.set_defaults(fn=cmd_init)
    a = sub.add_parser("check")
    a.add_argument("name", nargs="?", choices=NAMES + ["all"])
    a.add_argument("--alert", action="store_true")
    a.set_defaults(fn=cmd_check)
    a = sub.add_parser("pre-rebuild")
    a.add_argument("--alert", action="store_true")
    a.set_defaults(fn=cmd_pre_rebuild)
    sub.add_parser("post-rebuild").set_defaults(fn=cmd_post_rebuild)
    a = sub.add_parser("accept")
    a.add_argument("name", choices=NAMES)
    a.set_defaults(fn=cmd_accept)
    a = sub.add_parser("report")
    a.add_argument("name", choices=NAMES)
    a.set_defaults(fn=cmd_report)
    sub.add_parser("status").set_defaults(fn=cmd_status)
    args = p.parse_args()
    if os.geteuid() != 0 and os.environ.get("THORNS_AIDE_ALLOW_NONROOT") != "1":
        print("thorns-aide: run as root (sudo thorns-aide ...)", file=sys.stderr)
        sys.exit(2)
    sys.exit(args.fn(args))


if __name__ == "__main__":
    main()
