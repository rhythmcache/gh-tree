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
- ghtree
```
Usage: ghtree touch [OPTIONS] --repo <REPO> --output <OUTPUT>

Options:
      --pat <PAT>        GitHub Personal Access Token (can also use GH_TOKEN env var)
  -r, --repo <REPO>      Repository URL or owner/repo format
  -o, --output <OUTPUT>  Output directory path
  -b, --branch <BRANCH>  Branch name (default: repository's default branch)
  -h, --help             Print help
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

see [Usage](./usage.md)



## License

`ghtree` is licensed under the Apache License, Version 2.0. See [LICENSE](./LICENSE) for full details.

---

### Contributions

Contributions are welcome! Feel free to submit issues or pull requests on the [GitHub repository](https://github.com/rhythmcache/gh-tree).



