"""Archive completed numbered handset sessions outside the repository.

Existing files must match byte for byte. Never archive a running session.
"""
import argparse
import hashlib
import json
import shutil
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('/private/tmp'))
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--through', type=int, required=True,
                        help='Highest completed session number; never include a running session')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    if args.out.resolve().is_relative_to(root):
        raise SystemExit('Raw/private session archive must be outside the repository')
    files = []
    for directory in sorted(args.source.glob('ta1278-native-[0-9][0-9][0-9]')):
        if int(directory.name.rsplit('-', 1)[1]) > args.through:
            continue
        for source in sorted(directory.iterdir()):
            if source.is_symlink() or not source.is_file():
                raise SystemExit(f'Unexpected session entry: {source.name}')
            dest = args.out/directory.name/source.name
            if dest.exists() and sha(dest) != sha(source):
                raise SystemExit(f'Archive differs; no overwrite: {dest.name}')
            files.append((source, dest))
    # Preflight the entire copy before making changes.
    for source, dest in files:
        dest.parent.mkdir(parents=True, exist_ok=True)
        if not dest.exists():
            shutil.copy2(source, dest)
        if sha(source) != sha(dest):
            raise SystemExit(f'Copy verification failed: {dest.name}')
    records = [{'path': str(path.relative_to(args.out)), 'bytes': path.stat().st_size,
                'sha256': sha(path)} for path in sorted(args.out.rglob('*'))
               if path.is_file() and path.name != 'SHA256.json']
    (args.out/'SHA256.json').write_text(json.dumps({
        'private_local_only': True, 'files': records,
    }, indent=2)+'\n')
    print(f'Verified {len(files)} numbered-session files; archive contains {len(records)} files.')


if __name__ == '__main__':
    main()
