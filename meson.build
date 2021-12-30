project(
  'scallop', 'c',
  version : '0.0.1',
  license : 'GPL-3',
  meson_version : '>= 0.58.0',
)

bash_libdir = join_paths(meson.project_source_root(), 'bash')
bash_incdir = 'bash'

cc = meson.get_compiler('c')
dl_dep = cc.find_library('dl', required : true)
bash_dep = cc.find_library('bash', static : true, dirs : [bash_libdir], required : true)
bash_includes = include_directories([
  '.', bash_incdir,
  join_paths(bash_incdir, 'include'), join_paths(bash_incdir, 'builtins')])

subdir('src')
