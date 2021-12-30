#include <bash/builtins.h>
#include <bash/shell.h>
#include <bash/builtins/common.h>

int scallop() {
	parse_and_execute("PV=1.2", "", SEVAL_NOFREE);
	return parse_and_execute("PATH=/bin; echo PV=${PV}", "", SEVAL_NOFREE);
}
