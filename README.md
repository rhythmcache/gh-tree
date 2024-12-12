# github-repo-tree-viewer

A simple ⚠️experimental python script which allows you to view the tree (structure) of a public GitHub repository. 

- Install dependency
```
pip install requests
```

## Usage
- Download [github-tree.py](https://raw.githubusercontent.com/rhythmcache/github-repo-tree-viewer/refs/heads/main/github-tree.py?token=GHSAT0AAAAAAC3ZQLEH3O75QJ5LOHNA7SAOZ23BAIQ)
- Now run
```
python git-tree.py <GitHub Repository URL>
```
- This will show the tree of the repo

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


