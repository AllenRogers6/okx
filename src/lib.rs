pub mod db;

use mime_guess::mime;

pub fn is_text_file(path: &str) -> bool {
    let guess = mime_guess::from_path(path);
    if let Some(mime) = guess.first() {
        let type_ = mime.type_();
        let subtype = mime.subtype().as_str();
        if type_ == mime::TEXT {
            return true;
        }
        if type_ == mime::APPLICATION {
            return matches!(
                subtype,
                "json"
                    | "xml"
                    | "javascript"
                    | "x-javascript"
                    | "ecmascript"
                    | "csv"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "graphql"
                    | "sql"
                    | "x-sh"
                    | "x-httpd-php"
            );
        }
        return matches!(
            subtype,
            "html" | "css" | "markdown" | "x-markdown" | "plain"
        );
    }
    match std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
    {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "txt"
                | "md"
                | "rs"
                | "c"
                | "cpp"
                | "h"
                | "py"
                | "js"
                | "ts"
                | "java"
                | "go"
                | "html"
                | "css"
                | "json"
                | "xml"
                | "yaml"
                | "yml"
                | "toml"
                | "csv"
                | "ini"
                | "sh"
        ),
        None => false,
    }
}

pub fn open_with_app(path: &str) {
    if is_text_file(path) {
        let default_editor = if cfg!(target_os = "windows") {
            "notepad"
        } else {
            "vim"
        };

        let editor = std::env::var("FILERUN_EDITOR")
            .or_else(|_| std::env::var("EDITOR"))
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| default_editor.into());

        std::process::Command::new(&editor)
            .arg(path)
            .status()
            .unwrap_or_else(|_| panic!("failed to launch editor: {}", editor));
    } else {
        open::that(path).unwrap_or_else(|e| eprintln!("failed to open file: {e}"));
    }
}

pub fn interactive_pick(paths: &[String]) -> Option<String> {
    use dialoguer::FuzzySelect;

    let idx = FuzzySelect::new()
        .with_prompt("Select file (type to filter, ↑↓ to navigate, Enter to open)")
        .items(paths)
        .default(0)
        .interact_opt()
        .ok()??;

    Some(paths[idx].clone())
}

pub fn shell_init(shell: &str) -> &'static str {
    match shell {
        "bash" | "zsh" => {
            r#"
# Auto-tracking wrappers — silently add any file arg to okx before opening
_okx_add_file() {
  local cmd="$1"; shift
  for arg in "$@"; do
    [ -f "$arg" ] && okx add "$arg" 2>/dev/null &
  done
  command "$cmd" "$@"
}

for _cmd in vim nvim nano emacs code hx micro cat less bat; do
  eval "${_cmd}() { _okx_add_file ${_cmd} \"\$@\"; }"
done
unset _cmd

ok() {
  if [ $# -eq 0 ]; then command ok
  elif [ -f "$1" ]; then okx add "$1" 2>/dev/null; command ok "$1"
  else command ok "$@"
  fi
}
f() { ok "$@"; }
"#
        }
        "fish" => {
            r#"
function _okx_wrap
  set cmd $argv[1]
  set -e argv[1]
  for arg in $argv
    if test -f $arg
      okx add $arg 2>/dev/null &
    end
  end
  command $cmd $argv
end

for _cmd in vim nvim nano emacs code hx micro cat less bat
  eval "function $_cmd; _okx_wrap $_cmd \$argv; end"
end

function ok
  if test (count $argv) -eq 0; command ok
  else if test -f $argv[1]; okx add $argv[1] 2>/dev/null; command ok $argv
  else; command ok $argv
  end
end
function f; ok $argv; end
"#
        }
        "powershell" => {
            r#"
function _OkxAdd($path) { if (Test-Path $path -PathType Leaf) { okx add $path 2>$null } }

foreach ($c in @('vim','nvim','nano','code','notepad','cat')) {
  $fn = "function global:$c { _OkxAdd `$args[0]; & $c @args }"
  Invoke-Expression $fn
}

function ok {
  if ($args.Count -eq 0) { & ok }
  elseif (Test-Path $args[0] -PathType Leaf) { okx add $args[0] 2>$null; & ok @args }
  else { & ok @args }
}
function f { ok @args }
"#
        }
        _ => "# unsupported shell",
    }
}
