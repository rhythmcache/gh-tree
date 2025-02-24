## `ghtree` Usage

### Commands

#### 1. `touch`
Creates a placeholder directory structure based on the repository's tree.

**Usage:**
```bash
ghtree touch -r <GitHub Repository URL> -o <output-path> [-b <branch>]
```

**Arguments:**
- `-r, --repo <GitHub Repository URL>`: The URL of the GitHub repository.
- `-o, --output <output-path>`: The local directory where the placeholder structure will be created.
- `-b, --branch <branch>`: (Optional) The branch to use. If not provided, the default branch will be used.

**Example:**
```bash
ghtree touch -r https://github.com/user/repo -o ./my-repo -b main
```

#### 2. `view`
Displays the repository's directory structure in a tree format.

**Usage:**
```bash
ghtree view -r <GitHub Repository URL> [-b <branch>] [-f <folder>] [-c]
```

**Arguments:**
- `-r, --repo <GitHub Repository URL>`: The URL of the GitHub repository.
- `-b, --branch <branch>`: (Optional) The branch to use. If not provided, the default branch will be used.
- `-f, --folder <folder>`: (Optional) View a specific folder within the repository.
- `-c, --color`: (Optional) Enable colored output with icons.

**Example:**
```bash
ghtree view -r https://github.com/user/repo -b main -f src -c
```

#### 3. `pull`
Downloads a specific file or folder from the repository.

**Usage:**
```bash
ghtree pull -r <GitHub Repository URL> -f <file/folder to pull> [-b <branch>] [-o <output-directory>]
```

**Arguments:**
- `-r, --repo <GitHub Repository URL>`: The URL of the GitHub repository.
- `-f, --path <file/folder to pull>`: The file or folder to download.
- `-b, --branch <branch>`: (Optional) The branch to use.
- `-o, --output <output-directory>`: (Optional) The directory where the file/folder will be downloaded. If not provided, the current directory is used.

**Example:**
```bash
ghtree pull -r https://github.com/user/repo -f src -b main -o ./downloads
```

#### 4. `download`
Downloads the entire repository as a zip file.

**Usage:**
```bash
ghtree download -r <GitHub Repository URL> [-b <branch>] [-o <output-directory>]
```

**Arguments:**
- `-r, --repo <GitHub Repository URL>`: The URL of the GitHub repository.
- `-b, --branch <branch>`: (Optional) The branch to use. If not provided, the default branch will be used.
- `-o, --output <output-directory>`: (Optional) The directory where the zip file will be saved. If not provided, the current directory is used.

**Example:**
```bash
ghtree download -r https://github.com/user/repo -b main -o ./downloads
```

#### 5. `find`
Searches for a file within the repository.

**Usage:**
```bash
ghtree find -r <GitHub Repository URL> -f <filename> [-b <branch>] [--exact]
```

**Arguments:**
- `-r, --repo <GitHub Repository URL>`: The URL of the GitHub repository.
- `-f, --filename <filename>`: The name of the file to search for.
- `-b, --branch <branch>`: (Optional) The branch to search in. If not provided, all branches will be searched.
- `--exact`: (Optional) Enable exact filename matching.

**Example:**
```bash
ghtree find -r https://github.com/user/repo -f README.md -b main --exact
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
