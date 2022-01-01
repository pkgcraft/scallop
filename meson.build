project(
  'scallop', 'c',
  version : '0.0.1',
  license : 'GPL-3',
  meson_version : '>= 0.58.0',
)

bash_libdir = join_paths(meson.project_source_root(), 'bash')
bash_includes = include_directories([
  '.', 'bash', join_paths('bash', 'include'), join_paths('bash', 'builtins')])

cc = meson.get_compiler('c')
dl_dep = cc.find_library('dl', required : true)
bash_lib = cc.find_library('bash', static : true, dirs : [bash_libdir], required : true)

# meson doesn't support link_whole with external libraries 
# https://github.com/mesonbuild/meson/issues/4707
if host_machine.system() == 'darwin' and cc.get_id() == 'clang'
  iconv_dep = cc.find_library('iconv', required : true)
  intl_dep = cc.find_library('intl', required : true)
  bash_dep = declare_dependency(
    link_args : ['-force_load', join_paths(bash_libdir, 'libbash.a')],
    dependencies : [bash_lib, iconv_dep, intl_dep])
else
  bash_dep = declare_dependency(
    link_args : ['-Wl,--whole-archive', join_paths(bash_libdir, 'libbash.a'), '-Wl,--no-whole-archive'],
    dependencies : [bash_lib])
endif

incdir = include_directories(['src'])
subdir('src')
