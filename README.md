# gh-tree

A simple rust implementation which uses Github API to interact with repos directly from the terminal. Supports viewing and fetching repo structures, downloading files, and cloning entire trees as placeholders.

## Supported things

- View repository tree structure without cloning.

- Download specific files or folders from a repository.

- Generate a placeholder directory structure of repo

- Download an entire repo as a ZIP file directly

- also supports GitHub API authentication using Personal Access Token (PAT).

- others

## Usage

- See [Usage](./usage.md)
- ghtree
```
Usage: ghtree [OPTIONS] <COMMAND>

Commands:
  view        View repository structure
  touch       Create empty directory structure
  pull        Pull specific file or folder
  download    Download repository as zip
  find        Find file in repository
  help        Print this message or the help of the given subcommand(s)

Options:
  --pat <PAT>  GitHub Personal Access Token (can also use GH_TOKEN env var)
  -h, --help   Print help
  -V, --version Print version
```
- ghrls
 ```
ghrls view <user/repo or URL> [--tag <tag>] [-d/--detailed] [-n/--no-color] [--latest [N]]

ghrls pull <user/repo or URL> [--tag <tag>] (--all | <file>) [-o <dir>]
```
---

to use pat , you can also export it in environment
```
export GH_TOKEN=<your github PATH>
```

## Build
- Install Rust then run
```
git clone --depth 1 https://github.com/rhythmcache/gh-tree.git && cd gh-tree && chmod +x ./build.sh && ./build.sh
```





## License

`ghtree` is licensed under the Apache License, Version 2.0. See [LICENSE](./LICENSE) for full details.

---

### Contributions

Contributions are welcome! Feel free to submit issues or pull requests on the [GitHub repository](https://github.com/rhythmcache/gh-tree).



