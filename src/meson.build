libscallop = shared_library(
  'scallop', 'scallop.c',
  dependencies : [bash_dep, dl_dep],
  include_directories : [bash_includes],
  version : '5.1.12', soversion : '5',
)

demo = executable('demo', 'demo.c', link_with : libscallop)
