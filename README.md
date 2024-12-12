# github-repo-tree-viewer

- A Simple ⚠️ experimental Python script to recursively list the directory structure of GitHub Repositories. It Displays a Tree-Like format for easier visualization of Files and Directories similar to the `tree` Command for local directories on linux.

## Usage

- Ensure Python is installed in your System
- Install dependency
```
pip install requests
```
- Clone this repo or Download [github-tree.py](https://github.com/rhythmcache/github-repo-tree-viewer/releases/download/V1/github-tree.py)
- Now run
```
python github-tree.py <GitHub Repository URL>
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


