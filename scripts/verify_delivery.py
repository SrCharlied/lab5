"""Reproduce this delivery without private history or external assets.
Usage: python scripts/verify_delivery.py [--offline] [--window]
Logs and regenerated PNGs go to ignored target/delivery-verification/.
Reference image hashes were verified on Windows x64, not every platform/libm.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'target/delivery-verification'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def local_file(relative):
    path = Path(relative)
    if path.is_absolute() or '..' in path.parts:
        raise ValueError(f'Expected repository-relative file: {relative}')
    resolved = (ROOT / path).resolve()
    if not resolved.is_relative_to(ROOT):
        raise ValueError(f'File escapes the repository: {relative}')
    return resolved


def run(name, command, results):
    completed = subprocess.run(
        [str(part) for part in command], cwd=ROOT, text=True,
        encoding='utf-8', errors='replace', stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, timeout=300,
    )
    (OUT / f'{name}.log').write_text(completed.stdout, encoding='utf-8')
    results['commands'][name] = {'exit': completed.returncode}
    print(f'{name}: {completed.returncode}', flush=True)
    if completed.returncode:
        raise RuntimeError(f'{name} failed; see {OUT / (name + ".log")}')
    return completed.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--offline', action='store_true', help='Require cached Cargo dependencies')
    parser.add_argument('--window', action='store_true', help='Present three native frames per primary body')
    options = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((ROOT / 'docs/source-manifest.json').read_text(encoding='utf-8'))
    oracles = json.loads((ROOT / 'docs/render-oracles.json').read_text(encoding='utf-8'))
    actual_paths = {p.relative_to(ROOT).as_posix()
                    for folder in ('src', 'tests') for p in (ROOT / folder).rglob('*') if p.is_file()}
    expected_paths = {p for p in manifest['files'] if p.startswith(('src/', 'tests/'))}
    if actual_paths != expected_paths:
        raise RuntimeError('Source/test path set differs from the reviewed delivery')
    for relative, expected in manifest['files'].items():
        if digest(local_file(relative)) != expected:
            raise RuntimeError(f'Source/input hash mismatch: {relative}')
    for reference in oracles['png'] + oracles['media']:
        if digest(local_file(reference['file'])) != reference['sha256']:
            raise RuntimeError(f'Reference artifact changed: {reference["file"]}')
    results = {'commands': {}, 'source_files_checked': len(manifest['files']), 'renders': []}
    flags = ['--locked'] + (['--offline'] if options.offline else [])
    run('format', ['cargo', 'fmt', '--check'], results)
    tests = run('tests', ['cargo', 'test'] + flags, results)
    count = sum(map(int, re.findall(r'test result: ok\. (\d+) passed', tests)))
    if count != manifest['expected_tests']:
        raise RuntimeError(f'Unexpected passing test count: {count}')
    results['test_count'] = count
    run('clippy', ['cargo', 'clippy'] + flags + ['--all-targets', '--', '-D', 'warnings'], results)
    run('build', ['cargo', 'build', '--release'] + flags, results)
    executable = ROOT / 'target/release' / ('lab5.exe' if os.name == 'nt' else 'lab5')
    for index, reference in enumerate(oracles['png']):
        output = OUT / f'render-{index:02d}.png'
        output.unlink(missing_ok=True)
        arguments = reference['args']
        if not all(isinstance(value, str) for value in arguments):
            raise ValueError('Render arguments must be strings')
        if '--output' in arguments or '--headless' in arguments:
            raise ValueError('Reference arguments must not replace verification output controls')
        run(f'render-{index:02d}', [executable] + arguments + ['--headless', '--output', output], results)
        image = output.read_bytes()
        if image[:8] != b'\x89PNG\r\n\x1a\n':
            raise RuntimeError('Renderer did not write a PNG')
        dimensions = struct.unpack('>II', image[16:24])
        if dimensions != (reference['width'], reference['height']):
            raise RuntimeError(f'Unexpected image dimensions: {reference["file"]}')
        if digest(output) != reference['sha256']:
            raise RuntimeError(f'Render mismatch: {reference["file"]}; reference platform: Windows x64')
        results['renders'].append({'reference': reference['file'], 'sha256': digest(output)})
    if options.window:
        for name, arguments in (
            ('star', ['--neutron', '--angle', '-0.65']),
            ('rocky', ['--eyeball']), ('gas', ['--gas-v2']),
            ('pulsar', ['--pulsar', '--width', '640', '--height', '480']),
        ):
            run('window-' + name, [executable] + arguments + ['--frames', '3'], results)
    results['render_count'] = len(results['renders'])
    results['preserved_media_count'] = len(oracles['media'])
    (OUT / 'results.json').write_text(json.dumps(results, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'tests': count, 'exact_pngs': results['render_count'],
                      'media_checked': results['preserved_media_count']}, indent=2))


if __name__ == '__main__':
    main()
