## `ghtree` Usage

### Commands

#### 1. `touch`
Creates a placeholder directory structure based on the repository's tree.

**Usage:**
```bash
ghtree touch <output-path> <GitHub Repository URL> [branch]
```

**Arguments:**
- `<output-path>`: The local directory where the placeholder structure will be created.
- `<GitHub Repository URL>`: The URL of the GitHub repository.
- `[branch]`: (Optional) The branch to use. If not provided, the default branch will be used.

**Example:**
```bash
ghtree touch ./my-repo https://github.com/user/repo main
```

#### 2. `view`
Displays the repository's directory structure in a tree format.

**Usage:**
```bash
ghtree view <GitHub Repository URL> [branch] [-f <folder>] [-c]
```

**Arguments:**
- `<GitHub Repository URL>`: The URL of the GitHub repository.
- `[branch]`: (Optional) The branch to use. If not provided, the default branch will be used.
- `-f <folder>`: (Optional) View a specific folder within the repository.
- `-c`: (Optional) Enable colored output with icons.

**Example:**
```bash
ghtree view https://github.com/user/repo main -f src -c
```

#### 3. `pull`
Downloads a specific file or folder from the repository.

**Usage:**
```bash
ghtree pull [-o <output-directory>] <GitHub Repository URL> <branch> <file/folder to pull>
```

**Arguments:**
- `-o <output-directory>`: (Optional) The directory where the file/folder will be downloaded. If not provided, the current directory is used.
- `<GitHub Repository URL>`: The URL of the GitHub repository.
- `<branch>`: The branch to use.
- `<file/folder to pull>`: The file or folder to download.

**Example:**
```bash
ghtree pull -o ./downloads https://github.com/user/repo main src
```

#### 4. `-dl`
Downloads the entire repository as a zip file.

**Usage:**
```bash
ghtree -dl [-o <output-directory>] <GitHub Repository URL> [branch]
```

**Arguments:**
- `-o <output-directory>`: (Optional) The directory where the zip file will be saved. If not provided, the current directory is used.
- `<GitHub Repository URL>`: The URL of the GitHub repository.
- `[branch]`: (Optional) The branch to use. If not provided, the default branch will be used.

**Example:**
```bash
ghtree -dl -o ./downloads https://github.com/user/repo main
```

#### 5. `find`
Searches for a file within the repository.

**Usage:**
```bash
ghtree find <filename> <repo link> [branch] [--exact]
```

**Arguments:**
- `<filename>`: The name of the file to search for.
- `<repo link>`: The URL of the GitHub repository.
- `[branch]`: (Optional) The branch to search in. If not provided, all branches will be searched.
- `--exact`: (Optional) Enable exact filename matching.

**Example:**
```bash
ghtree find README.md https://github.com/user/repo main --exact
```

## `ghrls` Usage

### Commands

#### 1. `view`
Displays the list of releases and their assets.

**Usage:**
```bash
ghrls view <user/repo or URL> [--tag <tag>] [-d/--detailed] [-n/--no-color] [--latest [N]]
```

**Arguments:**
- `<user/repo or URL>`: The GitHub repository in the format `user/repo` or the full URL.
- `--tag <tag>`: (Optional) View a specific release by its tag.
- `-d/--detailed`: (Optional) Show detailed information about each asset.
- `-n/--no-color`: (Optional) Disable colored output.
- `--latest [N]`: (Optional) View the latest N releases. If N is not provided, it defaults to 1.

**Example:**
```bash
ghrls view user/repo --tag v1.0.0 -d
```

#### 2. `pull`
Downloads release assets.

**Usage:**
```bash
ghrls pull <user/repo or URL> [--tag <tag>] (--all | <file>) [-o <dir>]
```

**Arguments:**
- `<user/repo or URL>`: The GitHub repository in the format `user/repo` or the full URL.
- `--tag <tag>`: (Optional) Download assets from a specific release by its tag.
- `--all`: Download all assets from the release.
- `<file>`: Download a specific asset by its name.
- `-o <dir>`: (Optional) The directory where the assets will be downloaded. If not provided, the current directory is used.

**Example:**
```bash
ghrls pull user/repo --tag v1.0.0 --all -o ./downloads
```

#### 3. `--pat`
Authenticates with a GitHub Personal Access Token (PAT).

**Usage:**
```bash
ghrls --pat <token> [commands...]
```

**Arguments:**
- `<token>`: Your GitHub Personal Access Token.
- `[commands...]`: Any of the above commands.

**Example:**
```bash
ghrls --pat <your-token> view user/repo
```

## Common Options

- **GitHub Token**: Both tools support authentication via a GitHub Personal Access Token (PAT). You can provide the token using the `--pat` option or set it as an environment variable (`GH_TOKEN`).

## Examples

### Using `ghtree`
```bash
# Create a placeholder structure for a repository
ghtree touch ./my-repo https://github.com/user/repo main

# View the repository structure with colored output
ghtree view https://github.com/user/repo main -c

# Download a specific folder from the repository
ghtree pull -o ./downloads https://github.com/user/repo main src

# Search for a file in the repository
ghtree find README.md https://github.com/user/repo main --exact
```

### Using `ghrls`
```bash
# View the latest release with detailed information
ghrls view user/repo --latest -d

# Download all assets from a specific release
ghrls pull user/repo --tag v1.0.0 --all -o ./downloads

# Download a specific asset from the latest release
ghrls pull user/repo --latest my-asset.zip
```
