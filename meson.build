project('macro-manager', 'vala', 'c')

vapi_dir = meson.current_source_dir() / 'vapi'
add_project_arguments(['--vapidir', vapi_dir], language: 'vala')

dependencies = [
    dependency('glib-2.0'),
    dependency('gobject-2.0'),
    dependency('gio-2.0'),
    dependency('libxdo'),
    dependency('x11'),
    meson.get_compiler('c').find_library('libtoml'),
    meson.get_compiler('vala').find_library('posix'),
    meson.get_compiler('vala').find_library('libtoml', dirs: vapi_dir),
]

subdir('src')

executable('macro-manager', sources,
    dependencies: dependencies,
    install: true
)

pkg = import('pkgconfig')
lib = library('macro-manager', sources,
    dependencies: dependencies,
    install: true,
    install_dir: [true, true, true]
)
pkg.generate(lib)
