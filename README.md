# github-repo-tree-viewer

- A simple ⚠️ experimental Python script to recursively list the directory structure of GitHub repositories.

- It displays a tree-like format for easier visualization of files and directories, similar to the `tree` shell command for local directories on Linux.

-  the script can also create an exact placeholder structure of folders and files of a GitHub repository in your local directory.

- by default it uses public API access but it also supports GitHub `Personal Access Token (PAT) ` to fetch private repository and handle rate limits.

## Usage

- Ensure Python is installed in your System
- Install dependency
```
pip install requests
```
- Clone this repo or Download [github-tree.py](https://github.com/rhythmcache/github-repo-tree-viewer/releases/download/V2/github-tree.py)
- Now run
```
python github-tree.py view <GitHub Repository URL>
```
- This will show the tree of the repo
---

##### To show the tree of a branch of a repo , run
```
python github-tree.py <GitHub Repository URL> <Branch Name>
```
- This will show the tree of the specified Branch of repo

### Example
- The Command
```
python github-tree.py https://github.com/rhythmcache/video-to-bootanimation
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


##### Bugs
- Find and tell

---
[![Telegram](https://img.shields.io/badge/Telegram-Join%20Chat-blue?style=flat-square&logo=telegram)](https://t.me/ximistuffschat)


