#include <bash/builtins.h>
#include <bash/shell.h>
#include <bash/builtins/common.h>
#include <bash/builtins/builtext.h>

int scal_source_file(const char *path, char **env) {
	return source_file(path, 0);
}

int scal_dump_env(char **args) {
	WORD_LIST *list = strvec_to_word_list(args, 0, 0);
	return declare_builtin(list);
}
