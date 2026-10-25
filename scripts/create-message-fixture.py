#!/usr/bin/env python3
"""Create synthetic mailbox data in an unused test data directory; never overwrite."""
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--data-dir', type=Path, required=True)
args = parser.parse_args()
args.data_dir.mkdir(parents=True, exist_ok=True)
mailbox = {
    'drafts': [],
    'outbox': [{'to': '12345', 'body': '明天见。', 'time': 1792944000000, 'sent': True}],
    'inbox': [
        {'from': '12345', 'body': '这是一条本地测试信息。\n可以测试回复、转发和删除。', 'read': False, 'time': 1792944060000},
        {'from': '67890', 'body': '这条测试信息已读。\n' + '\n'.join(f'第 {i} 行：检查长正文滚动。' for i in range(1, 11)), 'read': True, 'time': 1792943940000},
    ],
}
try:
    with (args.data_dir / 'messages.json').open('x', encoding='utf-8') as stream:
        json.dump(mailbox, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
except FileExistsError:
    parser.exit(1, 'messages.json already exists; no changes made. Use a new test data directory.\n')
print(f'Created synthetic mailbox: {args.data_dir / "messages.json"}')
