#!/usr/bin/env python3
"""Reconstruct full z855 git history from ALL session logs (OpenClaw + Claude Code)."""
import json, os, sys, subprocess
from pathlib import Path

REPO = "/Users/matte/jeb"
BRANCH = "z855-full-reconstruction"

# === SOURCE DISCOVERY: both OpenClaw and Claude Code ===
all_session_files = set()

# 1. OpenClaw workspace session-logs
for f in Path('/Users/matte/.openclaw/workspace/session-logs').glob('*.jsonl'):
    all_session_files.add(str(f))

# 2. OpenClaw agent sessions
for f in Path('/Users/matte/.openclaw/agents').rglob('*.jsonl'):
    all_session_files.add(str(f))

# 3. Claude Code project sessions
for f in Path(os.path.expanduser('~/.claude/projects')).rglob('*.jsonl'):
    all_session_files.add(str(f))

print(f"Total session files found: {len(all_session_files)}")

def parse_ops(session_path):
    ops = []
    current_model = "unknown"
    try:
        lines = Path(session_path).read_text(errors='replace').splitlines()
    except:
        return ops

    for line in lines:
        line = line.strip()
        if not line: continue
        try:
            d = json.loads(line)
        except:
            continue

        dtype = d.get('type', '')

        # OpenClaw format
        if dtype == 'model_change':
            current_model = d.get('modelId', current_model)
            continue

        # Claude Code format: assistant turns have toolUse in content
        if dtype == 'message' or 'message' in d:
            msg = d.get('message', d)
            role = msg.get('role', '')
            if role != 'assistant':
                continue
            ts = d.get('timestamp', msg.get('timestamp', ''))
            model = msg.get('model', current_model)
            content = msg.get('content', [])
            if isinstance(content, str):
                continue

            for item in content:
                if not isinstance(item, dict):
                    continue
                itype = item.get('type', '')

                # OpenClaw uses 'toolCall', Claude Code uses 'tool_use'
                if itype not in ('toolCall', 'tool_use'):
                    continue

                name = item.get('name', '').lower()
                if name not in ('write', 'edit'):
                    continue

                # OpenClaw: arguments key; Claude Code: input key
                args = item.get('arguments', item.get('input', {}))
                path = args.get('file_path', args.get('path', ''))
                if not path:
                    continue

                # Only z855 crate files
                if '/z855/' not in path and '/z85' not in path.split('/')[-1].lower():
                    continue

                if name == 'write':
                    ops.append({'op':'write','path':path,'content':args.get('content',''),
                                'ts':ts,'model':model,'session':os.path.basename(session_path)})
                elif name == 'edit':
                    old = args.get('oldText', args.get('old_string', ''))
                    new = args.get('newText', args.get('new_string', ''))
                    if old:
                        ops.append({'op':'edit','path':path,'old':old,'new':new,
                                    'ts':ts,'model':model,'session':os.path.basename(session_path)})
        # Claude Code top-level format (no 'message' wrapper)
        elif d.get('role') == 'assistant':
            ts = d.get('timestamp', '')
            model = d.get('model', current_model)
            content = d.get('content', [])
            if not isinstance(content, list):
                continue
            for item in content:
                if not isinstance(item, dict): continue
                if item.get('type') != 'tool_use': continue
                name = item.get('name', '').lower()
                if name not in ('write', 'edit'): continue
                args = item.get('input', {})
                path = args.get('file_path', args.get('path', ''))
                if not path: continue
                if '/z855/' not in path and '/z85' not in path.split('/')[-1].lower():
                    continue
                if name == 'write':
                    ops.append({'op':'write','path':path,'content':args.get('content',''),
                                'ts':ts,'model':model,'session':os.path.basename(session_path)})
                elif name == 'edit':
                    old = args.get('old_str', args.get('oldText', args.get('old_string','')))
                    new = args.get('new_str', args.get('newText', args.get('new_string','')))
                    if old:
                        ops.append({'op':'edit','path':path,'old':old,'new':new,
                                    'ts':ts,'model':model,'session':os.path.basename(session_path)})
    return ops

print("Scanning for z855 ops...")
all_ops = []
sessions_with_ops = []
for s in sorted(all_session_files):
    ops = parse_ops(s)
    if ops:
        print(f"  {os.path.basename(s)}: {len(ops)} ops")
        all_ops.extend(ops)
        sessions_with_ops.append(s)

all_ops.sort(key=lambda o: o.get('ts', ''))
print(f"\nTotal: {len(all_ops)} z855 ops from {len(sessions_with_ops)} sessions")

if not all_ops:
    print("No ops found!")
    sys.exit(0)

from collections import Counter
print("\nFiles touched:")
for fname, count in sorted(Counter(op['path'].split('/')[-1] for op in all_ops).items(), key=lambda x: -x[1]):
    print(f"  {count:3d}x  {fname}")

print("\nBy model:")
for m, c in sorted(Counter(op['model'] for op in all_ops).items(), key=lambda x: -x[1]):
    print(f"  {c:3d}x  {m}")

timestamps = sorted(op['ts'] for op in all_ops if op['ts'])
if timestamps:
    print(f"\nDate range: {timestamps[0][:10]} → {timestamps[-1][:10]}")

def model_author(model):
    m = model.lower()
    if 'opus' in m: return ('Claude Opus', 'opus@anthropic.com')
    if 'sonnet' in m: return ('Claude Sonnet', 'sonnet@anthropic.com')
    if 'haiku' in m: return ('Claude Haiku', 'haiku@anthropic.com')
    return ('Claude', 'claude@anthropic.com')

def git(cmd, env=None):
    e = {**os.environ, **(env or {})}
    return subprocess.run(['git']+cmd, cwd=REPO, capture_output=True, text=True, env=e)

def git_env(ts, model):
    name, email = model_author(model)
    return {'GIT_AUTHOR_NAME':name,'GIT_AUTHOR_EMAIL':email,
            'GIT_AUTHOR_DATE':ts or '2026-02-13T00:00:00Z',
            'GIT_COMMITTER_DATE':ts or '2026-02-13T00:00:00Z',
            'GIT_COMMITTER_NAME':'Matte','GIT_COMMITTER_EMAIL':'matte@openclaw.local'}

# Delete existing branch if present
if BRANCH in git(['branch','--list', BRANCH]).stdout:
    git(['branch','-D', BRANCH])

orig = git(['rev-parse','--abbrev-ref','HEAD']).stdout.strip()
orig_sha = git(['rev-parse','HEAD']).stdout.strip()
print(f"\nCurrent branch: {orig} ({orig_sha[:8]})")
print(f"Creating orphan branch: {BRANCH}")

git(['checkout','--orphan', BRANCH])
git(['rm','-rf','--cached','.'])
subprocess.run(['git','clean','-fd'], cwd=REPO, capture_output=True)

first = all_ops[0]
env = {**os.environ, **git_env(first['ts'], first['model'])}
subprocess.run(['git','commit','--allow-empty','-m',
    f'z855 reconstruction: {len(all_ops)} ops from {len(sessions_with_ops)} sessions\n\n'
    f'Date range: {timestamps[0][:10]} → {timestamps[-1][:10]}\n\n'
    'Sources: OpenClaw agents + Claude Code projects\n\n'
    'Sessions:\n' + '\n'.join(f'  {os.path.basename(s)}' for s in sessions_with_ops)],
    cwd=REPO, env=env, capture_output=True)

committed = skipped = 0
for i, op in enumerate(all_ops):
    path = op['path']
    rel = path
    for prefix in [REPO+'/', '/Users/matte/jeb/']:
        if rel.startswith(prefix):
            rel = rel[len(prefix):]
            break
    abs_path = os.path.join(REPO, rel)
    os.makedirs(os.path.dirname(abs_path), exist_ok=True)
    env = {**os.environ, **git_env(op['ts'], op['model'])}

    if op['op'] == 'write':
        try:
            Path(abs_path).write_text(op['content'])
        except:
            skipped += 1; continue
        git(['add', rel])
        r = subprocess.run(['git','commit','-m',
            f"write: {rel}\n\nSession: {op['session']}\nModel: {op['model']}\nTime: {op['ts'][:19]}"],
            cwd=REPO, env=env, capture_output=True, text=True)
        if r.returncode == 0: committed += 1
        else: skipped += 1

    elif op['op'] == 'edit':
        if not os.path.exists(abs_path):
            skipped += 1; continue
        try:
            content = Path(abs_path).read_text(errors='replace')
            new_content = content.replace(op['old'], op['new'], 1)
            if new_content == content:
                skipped += 1; continue
            Path(abs_path).write_text(new_content)
        except:
            skipped += 1; continue
        git(['add', rel])
        r = subprocess.run(['git','commit','-m',
            f"edit: {rel}\n\nSession: {op['session']}\nModel: {op['model']}\nTime: {op['ts'][:19]}"],
            cwd=REPO, env=env, capture_output=True, text=True)
        if r.returncode == 0: committed += 1
        else: skipped += 1

    if (i+1) % 50 == 0:
        print(f"  Progress: {i+1}/{len(all_ops)} ({committed} committed, {skipped} skipped)")

commit_count = int(git(['rev-list','--count','HEAD']).stdout.strip() or 0)
print(f"\nDone: {committed} committed, {skipped} skipped, {commit_count} total commits on branch")

# Return to original
print(f"Returning to {orig}...")
r = subprocess.run(['git','checkout','-f', orig], cwd=REPO, capture_output=True, text=True)
if r.returncode != 0:
    subprocess.run(['git','checkout','-f', orig_sha], cwd=REPO, capture_output=True)

# Abort any existing merge first
subprocess.run(['git','merge','--abort'], cwd=REPO, capture_output=True)

print(f"Merging {BRANCH}...")
r = subprocess.run(['git','merge','--no-commit','--allow-unrelated-histories', BRANCH],
                   cwd=REPO, capture_output=True, text=True)
print("Exit code:", r.returncode)
if r.stdout.strip(): print(r.stdout[:500])
if r.stderr.strip(): print(r.stderr[:200])

print(f"\nBranch '{BRANCH}' has {commit_count} commits. In merge state.")
