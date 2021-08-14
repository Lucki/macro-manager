project('macro-manager', 'vala', 'c')

vapi_dir = meson.current_source_dir() / 'vapi'

add_project_arguments(['--vapidir', vapi_dir], language: 'vala')

dependencies = [
    dependency('glib-2.0'),
    dependency('gobject-2.0'),
    dependency('gio-2.0'),
    dependency('json-glib-1.0'),
    dependency('libxdo'),
    dependency('x11'),
    meson.get_compiler('vala').find_library('posix'),
]

subdir('src')

executable('macro-manager', sources, dependencies: dependencies)
library('macro-manager', sources, dependencies: dependencies)
