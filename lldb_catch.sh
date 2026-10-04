lldb -batch -n OxiReader -o "break set -n objc_exception_throw" -o "continue" -o "po \$arg1" -o "bt" -o "quit" > lldb_exception.txt
