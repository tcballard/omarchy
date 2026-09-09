import os
import pathlib
import selectors
import subprocess
import sys
import tempfile
import time

with tempfile.TemporaryDirectory() as temporary:
    base = pathlib.Path(temporary)
    root = base / 'plugins'
    root.mkdir()
    direct = root / 'direct'
    direct.mkdir()
    source = base / 'external'
    source.mkdir()
    (source / 'manifest.json').write_text('{}')
    (root / 'linked').symlink_to(source)
    process = subprocess.Popen([sys.argv[1], str(root)], stdout=subprocess.PIPE)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)

    def observe(action, expected):
        output = b''
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            action()
            if selector.select(.1):
                output += os.read(process.stdout.fileno(), 65536)
            if expected.encode() in output:
                return
        raise AssertionError((expected, output))

    try:
        observe(lambda: (direct / 'Direct.qml').write_text('edit'), '/direct/Direct.qml')
        observe(lambda: (source / 'Linked.qml').write_text('edit'), '/linked/Linked.qml')
        replacement = base / 'replacement'
        replacement.mkdir()
        (replacement / 'manifest.json').write_text('{}')
        (root / 'next').symlink_to(replacement)
        (root / 'next').replace(root / 'linked')
        observe(lambda: (replacement / 'Replacement.qml').write_text('edit'), '/linked/Replacement.qml')
        print('ok - direct, external linked and atomically retargeted source edits are observed')
    finally:
        process.terminate()
        process.wait(timeout=3)
        selector.close()
