docs_src = files(
    'librsvg.toml',
    'librsvg-r.svg',
    'librsvg.svg',
    'load-and-render.c',
    'migrating.md',
    'overview.md',
    'recommendations.md',
    'urlmap.js',
)

if build_gir.allowed()
    build_stamp = custom_target(
        'doc_build_stamp',
        command: [
            gidocgen,
            'generate',
            '-C',
            '@INPUT0@',
            '--content-dir',
            meson.current_source_dir(),
            '--output-dir',
            '@OUTDIR@',
            '@INPUT1@'
        ],
        input: files('librsvg.toml') + rsvg_gir[0],
        output: 'Rsvg-2.0',
        install: true,
        install_dir: get_option('datadir') / 'doc' / 'Rsvg-2.0'
    )
endif

custom_target('man',
    input: files(
        '../rsvg-convert.rst'
    ),
    output: '@BASENAME@.1',
    command: [
        rst2man,
        '--syntax-highlight=none',
        '@INPUT@',
        '@OUTPUT@'
    ],
    install: true,
    install_dir: get_option('mandir') / 'man1',
)