import json, os, pathlib, shlex, subprocess, tempfile, time

base = pathlib.Path(__file__).resolve().parents[1] / 'target' / 'smoke'
base.mkdir(parents=True, exist_ok=True)
exe = os.environ.get('GLANCE_TEST_BINARY', str(base.parent / 'debug' / 'glance-panel'))
socket = f'glance-qa-{os.getpid()}'
def tmux(*args):
    return subprocess.check_output(['tmux', '-L', socket, *args], text=True)
def until(check):
    end = time.monotonic() + 8
    while time.monotonic() < end:
        value = check()
        if value: return value
        time.sleep(.1)
    raise AssertionError('Timed out waiting for terminal state')
with tempfile.TemporaryDirectory(prefix='tmux-', dir=base) as tmp:
    root = pathlib.Path(tmp)
    state = root / 'state'; state.mkdir()
    claude = root / 'claude'; project = claude / 'projects/test'; project.mkdir(parents=True)
    (state / 'config.json').write_text(json.dumps({'hook_offer':'declined','no_model':True}))
    (project / 'session.jsonl').write_text(json.dumps({'type':'user','cwd':str(root),'message':{'content':'Verify the split and todo controls.'}})+'\n')
    try:
        tmux('new-session', '-d', '-s', 'glance', '-x', '160', '-y', '40', 'env', f'GLANCE_HOME={state}', f'CLAUDE_CONFIG_DIR={claude}', 'bash', '--noprofile', '--norc')
        source = tmux('list-panes', '-F', '#{pane_id}').strip()
        tmux('send-keys', '-t', source, '-l', shlex.join([exe, '--no-model', 'attach', '--backend', 'tmux', '--session', 'session', '--ratio', '0.45']))
        tmux('send-keys', '-t', source, 'Enter')
        panel = until(lambda: next((line.split()[0] for line in tmux('list-panes','-F','#{pane_id} #{pane_current_command}').splitlines() if 'glance-panel' in line), None))
        until(lambda: 'WHAT WE ARE WORKING ON' in tmux('capture-pane','-p','-t',panel))
        tmux('send-keys','-t',panel,'-l','aRemember to verify keyboard input')
        tmux('send-keys','-t',panel,'Enter')
        todo = state / 'session.todos.json'
        until(lambda: todo.exists() and len(json.loads(todo.read_text())['items']) == 1)
        assert json.loads(todo.read_text())['items'][0]['text'] == 'Remember to verify keyboard input'
        tmux('send-keys','-t',panel,'-l','x')
        until(lambda: json.loads(todo.read_text())['items'][0]['status'] == 'done')
        captured = tmux('capture-pane','-p','-t',panel)
        (base / 'tmux-todos.txt').write_text(captured)
        assert 'MY TODOS' in captured
        tmux('send-keys','-t',panel,'-l','d')
        until(lambda: not json.loads(todo.read_text())['items'])
        tmux('send-keys','-t',panel,'-l','q')
        print('Real tmux split: attach, render, add, toggle, delete and quit passed.')
    except Exception:
        for pane in tmux('list-panes','-F','#{pane_id}').splitlines():
            print(tmux('capture-pane','-p','-t',pane))
        raise
    finally:
        subprocess.run(['tmux','-L',socket,'kill-server'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
