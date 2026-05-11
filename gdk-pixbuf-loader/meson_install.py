#!/usr/bin/env python3

from argparse import ArgumentParser
from pathlib import Path
import os
import subprocess
import sys

if __name__ == '__main__':
    argparse = ArgumentParser(description='Deploy loaders.cache. Pass required paths, or --show-cross-message.')
    argparse.add_argument('--queryloaders', type=Path, metavar="PATH", help="gdk-pixbuf-queryloaders to run")
    argparse.add_argument('--moduledir', type=Path, metavar="PATH", help="installed gdk-pixbuf module directory")
    argparse.add_argument('--cache-file', type=Path, metavar="PATH", help="module cache file to write")
    argparse.add_argument('--show-cross-message', action='store_true', help="tell the user to run query-loaders manually")
    args = argparse.parse_args()

    if not(args.show_cross_message or (args.queryloaders and args.moduledir and args.cache_file)):
        argparse.print_help()
        sys.exit(1)

    if args.show_cross_message or os.environ.get("DESTDIR"):
        print('*** Note: Please run gdk-pixbuf-queryloaders manually ' +
              'against the newly-built gdkpixbuf-svg loader', file=sys.stderr)
    else:
        env = os.environ.copy()
        env['GDK_PIXBUF_MODULEDIR'] = args.moduledir.as_posix()
        with args.cache_file.open('w', encoding='utf-8') as f:
            subprocess.run(
                [args.queryloaders.as_posix()],
                env=env,
                stdout=f,
                check=True
            )
