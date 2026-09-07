"""Validate release archives and real installers without model calls or user setup."""
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from threading import Thread
import zipfile


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


def exercise(binary, directory):
    env = {**os.environ, 'GLANCE_HOME': str(directory / 'state')}
    version = subprocess.check_output([str(binary), '--version'], env=env, text=True, timeout=30).strip()
    assert version.startswith('glance-panel '), version
    fixture = Path(__file__).parent / 'fixtures' / 'codex.jsonl'
    result = subprocess.check_output([str(binary), '--harness', 'codex', '--no-model', '--transcript', str(fixture.resolve()), 'transcript', '--session', 'same-id'], env=env, text=True, timeout=30)
    assert len(json.loads(result)['turns']) == 3
    subprocess.run([str(binary), 'todo', 'Verify installation', '--session', 'smoke'], env=env, check=True, timeout=30, stdout=subprocess.DEVNULL)
    return version


def install(root, directory, name):
    installer = root / ('glance-panel-installer.ps1' if os.name == 'nt' else 'glance-panel-installer.sh')
    destination = directory / 'install with spaces'
    home = directory / 'installer-home'
    home.mkdir()
    claude = home / '.claude'
    claude.mkdir()
    settings = claude / 'settings.json'
    settings.write_text('{"existing":true}\n')
    cursor = home / '.cursor'
    cursor.mkdir()
    hooks = cursor / 'hooks.json'
    hooks.write_text('{"version":1,"hooks":{}}\n')
    before = {path: path.read_bytes() for path in (settings, hooks)}
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(root.resolve())))
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        env = {key: value for key, value in os.environ.items() if not key.startswith(('GLANCE_PANEL_', 'INSTALLER_', 'CARGO_DIST_'))}
        # Python can inherit PowerShell 7's module path, which breaks Windows
        # PowerShell 5.1 auto-loading. Let the child select its own modules.
        env.pop('PSMODULEPATH', None)
        env.pop('PSModulePath', None)
        env.update({
            'GLANCE_PANEL_DOWNLOAD_URL': f'http://127.0.0.1:{server.server_port}',
            'GLANCE_PANEL_UNMANAGED_INSTALL': str(destination),
            'HOME': str(home), 'USERPROFILE': str(home),
            'XDG_CONFIG_HOME': str(home / '.config'),
            'CLAUDE_CONFIG_DIR': str(claude),
            'GLANCE_HOME': str(home / 'glance'),
        })
        command = ['powershell.exe', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File'] if os.name == 'nt' else ['sh']
        subprocess.run([*command, str(installer.resolve())], env=env, check=True, timeout=120)
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
    for path, content in before.items():
        assert path.read_bytes() == content, f'installer modified {path.name}'
    assert not (home / 'glance').exists(), 'installer must not configure Glance or hooks'
    return exercise(destination / name, directory / 'installed-check')


def check(root, target):
    extension = '.zip' if 'windows' in target else '.tar.xz'
    archive = root / f'glance-panel-{target}{extension}'
    expected = archive.with_name(archive.name + '.sha256').read_text().split()[0]
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    assert actual.lower() == expected.lower(), 'archive checksum mismatch'
    with tempfile.TemporaryDirectory(prefix='glance-release-smoke-') as scratch:
        directory = Path(scratch)
        if extension == '.zip':
            with zipfile.ZipFile(archive) as source:
                for member in source.infolist():
                    destination = (directory / member.filename).resolve()
                    assert destination.is_relative_to(directory.resolve()), 'archive path escaped extraction directory'
                source.extractall(directory)
        else:
            with tarfile.open(archive) as source:
                source.extractall(directory, filter='data')
        name = 'glance-panel.exe' if 'windows' in target else 'glance-panel'
        binaries = list(directory.rglob(name))
        assert len(binaries) == 1, f'expected one binary, found {binaries}'
        binary = binaries[0]
        if extension != '.zip':
            binary.chmod(0o700)
        version = exercise(binary, directory / 'archive-check')
        assert install(root, directory, name) == version, 'installer version differs from archive'
        print(f'{target}: checksum, extraction, {version}, native installer, transcript and todo checks passed')


if __name__ == '__main__':
    check(Path(sys.argv[1]), os.environ['TARGET'])
