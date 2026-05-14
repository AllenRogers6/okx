# f - fuzzy open files tracked by filerun
f() {
  if [ $# -eq 0 ]; then
    echo "Usage: f <pattern>"
    return 1
  fi
  # If argument is an existing file, add it and open
  if [ -f "$1" ]; then
    okx add "$1"
    okx open "$1"
  else
    open "$1"
  fi
}
