# github-repo-tree-viewer

- A simple binary which uses github api  to recursively list the directory structure of GitHub repositories.

- It displays a tree-like format for easier visualization of files and directories, similar to the `tree` shell command for local directories on Linux.

-  it can also create a placeholder structure of folders and files of a GitHub repository in your local directory.

- by default it uses public API access but it also supports GitHub `Personal Access Token (PAT) ` to fetch private repository and handle rate limits.

## Usage
Usage:
```
gh-tree touch <output-path> <GitHub Repository URL> [branch]
```
it will create local placeholders of files and folders of github repos 
 
```
gh-tree view <GitHub Repository URL> [branch]
```
will show the tree of the repo in fhe terminal
---



##### To show the tree of a branch of a repo , run
```
gh-tree view <GitHub Repository URL> <Branch Name>
```
- This will show the tree of the specified Branch of repo
---
To create local placeholders of files and folders of a branch of a github repo , run
```
gh-tree touch <output-path> <GitHub Repository URL> <Branch>
```

### Example
- The Command
```
gh-tree view https://github.com/rhythmcache/video-to-bootanimation
```
- The Output
```
├── META-INF
│   └── com
│       └── google
│           └── android
│               ├── update-binary
│               └── updater-script
├── README.md
├── bin
│   ├── ffmpeg
│   └── zip
├── customize.sh
└── module.prop

Total folders: 5
Total files: 7
```

## Private Repos
To view the tree, create placeholders of your private repo, or handle public API rate limit restrictions, you can use this as arguement 
```
--pat <yout token>
```

##### Bugs
- Find and tell

---
[![Telegram](https://img.shields.io/badge/Telegram-Join%20Chat-blue?style=flat-square&logo=telegram)](https://t.me/ximistuffschat)


