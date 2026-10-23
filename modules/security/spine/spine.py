#!/usr/bin/env python3
"""SPINE - local security event collector and notifier (v0.1).

Modes:
  spine daemon   root service: reads the audit log, ClamAV journal, kernel state
  spine notify   user service: turns new events into desktop notifications
  spine status   overview (default)
  spine log      recent events
  spine test     (needs sudo) write a test event to check the pipeline
  spine event    (needs sudo) write an event; used by other tools (thorns-aide)

SPINE only observes and reports. It never blocks, kills or deletes anything.
"""
import argparse
import grp
import html
import json
import os
import pwd
import re
import subprocess
import sys
import threading
import time

VERSION = "0.1.0"
STATE_DIR = "/var/lib/spine"
EVENTS = os.path.join(STATE_DIR, "events.jsonl")
STATE = os.path.join(STATE_DIR, "state.json")
AUDIT_LOG = "/var/log/audit/audit.log"
BOOTED_KERNEL = "/run/booted-system/kernel"
LATEST_KERNEL = "/nix/var/nix/profiles/system/kernel"
CLAM_UNITS = [
    "clamav-daemon.service",
    "clamav-clamonacc.service",
    "clamav-weekly-scan.service",
]
POLL_SECONDS = 5
SEV_ORDER = ["crit", "warn", "notice", "info"]
STARTED = int(time.time())

_lock = threading.Lock()
_last_seen = {}
_stats = {"suppressed": 0, "module_loads": 0}


# ----------------------------------------------------------------- events

def emit(sev, source, title, detail="", dedupe=None, window=600):
    """Append one event to the log, unless an identical one was just seen."""
    now = time.time()
    with _lock:
        if dedupe is not None:
            prev = _last_seen.get(dedupe)
            if prev is not None and now - prev < window:
                _stats["suppressed"] += 1
                return
            _last_seen[dedupe] = now
            if len(_last_seen) > 2000:
                for k in [k for k, v in _last_seen.items() if now - v > 86400]:
                    del _last_seen[k]
        rec = {"ts": int(now), "sev": sev, "src": source,
               "title": title, "detail": detail}
        with open(EVENTS, "a", encoding="utf-8") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")


def trim_events():
    try:
        with open(EVENTS, "r", encoding="utf-8") as f:
            lines = f.readlines()
    except OSError:
        return
    if len(lines) > 10000:
        with open(EVENTS, "w", encoding="utf-8") as f:
            f.writelines(lines[-5000:])


def write_state(kernel):
    st = {
        "version": VERSION,
        "started": STARTED,
        "heartbeat": int(time.time()),
        "kernel": kernel,
        "module_loads": _stats["module_loads"],
        "suppressed": _stats["suppressed"],
        "audit_log": os.path.exists(AUDIT_LOG),
    }
    tmp = STATE + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(st, f)
    try:
        os.chown(tmp, 0, grp.getgrnam("wheel").gr_gid)
    except (KeyError, OSError):
        pass
    os.chmod(tmp, 0o640)
    os.replace(tmp, STATE)


# ------------------------------------------------------------ audit parsing

REC = re.compile(r"^type=(\w+) msg=audit\((\d+)\.\d+:(\d+)\): (.*)$")
KV = re.compile(r'(\w+)=("[^"]*"|\S*)')


def kv(text):
    out = {}
    for m in KV.finditer(text):
        out.setdefault(m.group(1), m.group(2))
    return out


def dec(v):
    """Decode an audit field: quoted text, or hex when it had odd characters."""
    if v is None or v == "(null)":
        return ""
    if len(v) >= 2 and v[0] == '"' and v[-1] == '"':
        return v[1:-1]
    if v and len(v) % 2 == 0 and re.fullmatch(r"[0-9A-Fa-f]+", v):
        try:
            return bytes.fromhex(v).decode("utf-8", "replace").replace("\x00", " ")
        except ValueError:
            return v
    return v


def uname(v):
    try:
        n = int(v)
        if n == 4294967295:
            return "unset"
        return pwd.getpwuid(n).pw_name
    except (ValueError, KeyError, TypeError):
        return str(v)


def handle_audit(ev):
    sysc = None
    paths, parents = [], []
    for rtype, rest in ev["recs"]:
        f = kv(rest)
        if rtype == "SYSCALL":
            sysc = f
        elif rtype == "PATH":
            (parents if f.get("nametype") == "PARENT" else paths).append(dec(f.get("name")))
    if sysc is None:
        return
    key = sysc.get("key", "").strip('"')
    if key in ("", "(null)"):
        return

    comm = dec(sysc.get("comm"))
    exe = dec(sysc.get("exe"))
    who = "%s/%s" % (uname(sysc.get("auid")), uname(sysc.get("uid")))
    target = (paths or parents or ["?"])[0]
    failed = sysc.get("success") == "no"
    from_store = exe.startswith("/nix/store/")
    detail = "%s (%s) by %s: %s" % (comm, exe, who, target)

    if key == "kernel-modules":
        # udev/modprobe loads are expected and numerous: only count them.
        _stats["module_loads"] += 1
        if from_store:
            return
        emit("crit", "audit:" + key, "Kernel module loaded outside the Nix store",
             detail, dedupe=(key, exe), window=3600)
        return

    rules = {
        "ssh": ("/root/.ssh changed", "warn" if from_store else "crit"),
        "boot": ("Boot partition changed", "notice" if from_store else "crit"),
        "nocer": ("NOCER state changed", "notice"),
        "autostart": ("Autostart entry changed", "warn"),
        "user-systemd": ("User systemd unit changed", "warn"),
    }
    title, sev = rules.get(key, ("Audit event: " + key, "notice"))
    if failed:
        title += " (denied)"
        if sev == "crit":
            sev = "warn"
    dkey = (key, "" if key == "boot" else target)
    emit(sev, "audit:" + key, title, detail, dedupe=dkey, window=600)


class AuditTail:
    """Follows audit.log and groups records into events by serial number."""

    def __init__(self):
        self.fh = None
        self.inode = None
        self.buf = ""
        self.pending = {}
        self.cycle = 0

    def _new_text(self):
        try:
            st = os.stat(AUDIT_LOG)
        except OSError:
            return ""
        if self.fh is None or st.st_ino != self.inode:
            first = self.inode is None
            if self.fh:
                self.fh.close()
            self.fh = open(AUDIT_LOG, "rb")
            self.inode = st.st_ino
            if first:
                self.fh.seek(0, os.SEEK_END)  # skip history on first start
        elif st.st_size < self.fh.tell():
            self.fh.seek(0)
        return self.fh.read().decode("utf-8", "replace")

    def feed(self, lines):
        for line in lines:
            m = REC.match(line)
            if not m:
                continue
            rtype, ts, serial, rest = m.groups()
            ev = self.pending.setdefault(serial, {"recs": [], "cycle": 0})
            ev["recs"].append((rtype, rest))
            ev["cycle"] = self.cycle
        # An event is complete once a whole poll cycle passed without new records.
        for serial in [s for s, e in self.pending.items() if e["cycle"] < self.cycle]:
            handle_audit(self.pending.pop(serial))

    def poll(self):
        self.cycle += 1
        self.buf += self._new_text()
        lines = self.buf.split("\n")
        self.buf = lines.pop()
        self.feed(lines)


# ------------------------------------------------------------------ sources

def clam_thread():
    cmd = ["journalctl", "-f", "-n", "0", "-o", "json"]
    for unit in CLAM_UNITS:
        cmd += ["-u", unit]
    while True:
        try:
            proc = subprocess.Popen(cmd, stdout=subprocess.PIPE,
                                    stderr=subprocess.DEVNULL, text=True)
            for line in proc.stdout:
                try:
                    msg = json.loads(line).get("MESSAGE", "")
                except ValueError:
                    continue
                if not isinstance(msg, str):
                    continue
                msg = msg.strip()
                if msg.endswith(" FOUND") and ": " in msg:
                    emit("crit", "clamav", "Malware detected", msg,
                         dedupe=("clam", msg), window=3600)
            proc.wait()
        except OSError:
            pass
        time.sleep(30)


def kernel_version(path):
    m = re.search(r"linux[\w-]*?-(\d[\w.]*)/", path)
    return m.group(1) if m else "?"


def check_kernel():
    if not (os.path.exists(BOOTED_KERNEL) and os.path.exists(LATEST_KERNEL)):
        return None
    booted = os.path.realpath(BOOTED_KERNEL)
    latest = os.path.realpath(LATEST_KERNEL)
    info = {"running": kernel_version(booted),
            "installed": kernel_version(latest),
            "pending": booted != latest}
    if info["pending"]:
        emit("warn", "kernel", "Kernel update pending: reboot to apply",
             "running %s, installed %s" % (info["running"], info["installed"]),
             dedupe=("kernel", latest), window=86400)
    return info


# ------------------------------------------------------------------- daemon

def run_daemon():
    trim_events()
    emit("info", "spine", "SPINE daemon started", "version " + VERSION)
    threading.Thread(target=clam_thread, daemon=True).start()
    tail = AuditTail()
    last_slow = 0.0
    while True:
        try:
            tail.poll()
        except Exception as e:  # keep running whatever happens
            emit("warn", "spine", "Audit reader error", repr(e),
                 dedupe=("audit-error",), window=3600)
        now = time.time()
        if now - last_slow >= 60:
            last_slow = now
            try:
                write_state(check_kernel())
            except Exception as e:
                emit("warn", "spine", "State update error", repr(e),
                     dedupe=("state-error",), window=3600)
        time.sleep(POLL_SECONDS)


# ------------------------------------------------------------------- notify

URGENCY = {"notice": "low", "warn": "normal", "crit": "critical"}
ICON = {"notice": "dialog-information", "warn": "dialog-warning", "crit": "dialog-error"}


def deliver(rec):
    sev = rec.get("sev")
    if sev not in URGENCY:
        return
    subprocess.run(
        ["notify-send", "-a", "SPINE", "-u", URGENCY[sev], "-i", ICON[sev],
         "SPINE [%s] %s" % (sev.upper(), rec.get("title", "")),
         html.escape(rec.get("detail", ""))],
        check=False)


def run_notify():
    state_home = os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state")
    off_path = os.path.join(state_home, "spine", "offset")
    os.makedirs(os.path.dirname(off_path), exist_ok=True)
    try:
        with open(off_path) as f:
            offset = int(f.read().strip())
    except (OSError, ValueError):
        offset = None
    while True:
        try:
            size = os.path.getsize(EVENTS)
        except OSError:
            time.sleep(5)
            continue
        if offset is None or offset > size:
            offset = size  # first run or log trimmed: do not replay history
            with open(off_path, "w") as f:
                f.write(str(offset))
        if size > offset:
            with open(EVENTS, "rb") as f:
                f.seek(offset)
                data = f.read(size - offset)
            cut = data.rfind(b"\n") + 1
            for line in data[:cut].decode("utf-8", "replace").splitlines():
                try:
                    deliver(json.loads(line))
                except ValueError:
                    pass
            if cut:
                offset += cut
                with open(off_path, "w") as f:
                    f.write(str(offset))
        time.sleep(3)


# ---------------------------------------------------------------------- CLI

def load_events():
    out = []
    try:
        with open(EVENTS, encoding="utf-8") as f:
            for line in f:
                try:
                    out.append(json.loads(line))
                except ValueError:
                    pass
    except OSError as e:
        print("spine: cannot read %s: %s" % (EVENTS, e), file=sys.stderr)
    return out


def fmt_time(ts):
    return time.strftime("%Y-%m-%d %H:%M", time.localtime(ts))


def print_event(e):
    print("%s  %-6s %-20s %s" % (fmt_time(e.get("ts", 0)), e.get("sev", "?"),
                                 e.get("src", "?"), e.get("title", "")))
    if e.get("detail"):
        print(" " * 46 + e["detail"])


def cmd_status():
    try:
        with open(STATE, encoding="utf-8") as f:
            st = json.load(f)
    except (OSError, ValueError):
        st = {}
    print("SPINE %s" % VERSION)
    age = int(time.time()) - st.get("heartbeat", 0) if st else None
    if age is not None and age < 180:
        print("daemon     : running (heartbeat %ds ago)" % age)
    else:
        print("daemon     : NOT RUNNING (check: systemctl status spined)")
    print("audit log  : %s" % ("readable by daemon" if st.get("audit_log") else "not found"))
    k = st.get("kernel")
    if k:
        if k.get("pending"):
            print("kernel     : REBOOT NEEDED (running %s, installed %s)"
                  % (k["running"], k["installed"]))
        else:
            print("kernel     : up to date (%s)" % k["running"])
    events = load_events()
    day = time.time() - 86400
    counts = {s: 0 for s in SEV_ORDER}
    for e in events:
        if e.get("ts", 0) >= day and e.get("sev") in counts:
            counts[e["sev"]] += 1
    print("last 24h   : " + "  ".join("%s %d" % (s, counts[s]) for s in SEV_ORDER))
    print("counters   : %d module loads seen, %d duplicate events suppressed"
          % (st.get("module_loads", 0), st.get("suppressed", 0)))
    important = [e for e in events if e.get("sev") != "info"][-5:]
    if important:
        print("\nrecent events:")
        for e in important:
            print_event(e)


def cmd_log(n):
    for e in load_events()[-n:]:
        print_event(e)


def cmd_test():
    try:
        emit("notice", "test", "SPINE test event",
             "Pipeline check: log -> notification")
    except PermissionError:
        print("spine: run with sudo: sudo spine test", file=sys.stderr)
        sys.exit(1)
    print("test event written; a notification should appear within a few seconds")


def cmd_event(args):
    try:
        emit(args.sev, args.src, args.title, args.detail)
    except PermissionError:
        print("spine: run with sudo: sudo spine event ...", file=sys.stderr)
        sys.exit(1)


def main():
    p = argparse.ArgumentParser(prog="spine", description="SPINE security event watcher")
    sub = p.add_subparsers(dest="cmd")
    sub.add_parser("daemon", help="run the collector (root service)")
    sub.add_parser("notify", help="run desktop notifications (user service)")
    sub.add_parser("status", help="show overview")
    lg = sub.add_parser("log", help="show recent events")
    lg.add_argument("-n", type=int, default=20)
    sub.add_parser("test", help="write a test event (needs sudo)")
    ev = sub.add_parser("event", help="write an event (needs sudo)")
    ev.add_argument("sev", choices=["info", "notice", "warn", "crit"])
    ev.add_argument("src")
    ev.add_argument("title")
    ev.add_argument("detail", nargs="?", default="")
    args = p.parse_args()
    if args.cmd == "daemon":
        run_daemon()
    elif args.cmd == "notify":
        run_notify()
    elif args.cmd == "log":
        cmd_log(args.n)
    elif args.cmd == "test":
        cmd_test()
    elif args.cmd == "event":
        cmd_event(args)
    else:
        cmd_status()


if __name__ == "__main__":
    main()
