# gh-tree

A simple rust implementation which uses Github API to interact with repos directly from the terminal. Supports viewing and fetching repo structures, downloading files, and cloning entire trees as placeholders.

## Supported things

- View repository tree structure without cloning.

- Download specific files or folders from a repository.

- Generate a placeholder directory structure of repo

- Download an entire repo as a ZIP file directly

- also supports GitHub API authentication using Personal Access Token (PAT).

## Usage
```
ghtree touch <output-path> <GitHub Repository URL> [branch]
  ghtree view <GitHub Repository URL> [branch] [-f <folder>] [-c]
  ghtree pull [-o <output-directory>] <GitHub Repository URL> [branch] <file/folder to pull>
  ghtree -dl [-o <output-directory>] <GitHub Repository URL> [branch]
  ghtree --pat <token> [commands...]

Options:
  -c    Enable colored output with icons
  -f    View a specific folder in the repository
```
---
### View a GitHub Repository Tree
```sh
ghtree view <GitHub Repository URL> [branch]
```

**Example:**

```sh
ghtree view https://github.com/torvalds/linux
```

To view the tree of a specific folder within a repository:

```sh
ghtree view https://github.com/torvalds/linux -f kernel
```

### Generate a Placeholder Directory Structure

```sh
ghtree touch <output-path> <GitHub Repository URL> [branch]
```

**Example:**

```sh
ghtree touch ./linux-tree https://github.com/torvalds/linux main
```

### Download a Specific File or Folder

```sh
ghtree pull [-o <output-directory>] <GitHub Repository URL> [branch] <file/folder path>
```

**Example:**

```sh
ghtree pull -o ./linux https://github.com/torvalds/linux master kernel/sched
```

This downloads `kernel/sched` from the Linux repository into `./linux`.

### Download an Entire Repository as a ZIP File

```sh
ghtree -dl [-o <output-directory>] <GitHub Repository URL> [branch]
```

**Example:**

```sh
ghtree -dl -o /home/winter/Documents https://github.com/torvalds/linux
```

---

## Authentication
This implementation uses github public api to fetch things
GitHub imposes rate limits on unauthenticated api requests. to increase request limits or to make it work with your private repos use a **Personal Access Token (PAT):**

```sh
ghtree --pat <your_token> view https://github.com/torvalds/linux
```

Alternatively, set the token as an environment variable for persistent authentication:

```sh
export GH_TOKEN=<your_token>
```

With this set, you can run commands without needing to specify `--pat` each time.

---

## License

`ghtree` is licensed under the Apache License, Version 2.0. See [LICENSE](./LICENSE) for full details.

---

### Contributions

Contributions are welcome! Feel free to submit issues or pull requests on the [GitHub repository](https://github.com/rhythmcache/gh-tree).



