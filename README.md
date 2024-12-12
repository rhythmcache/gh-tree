# github-repo-tree-viewer

A simple ⚠️experimental python script which allows you to view the tree (structure) of a public GitHub repository. 

- Install dependency
```
pip install requests
```

## Usage
- Download `github-tree.py`
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
└── META-INF
├── META-INF
│   └── com
├── META-INF
│   ├── com
│   │   └── google
├── META-INF
│   ├── com
│   │   ├── google
│   │   │   └── android
├── META-INF
│   ├── com
│   │   ├── google
│   │   │   ├── android
│   │   │   │   └── update-binary
├── META-INF
│   ├── com
│   │   ├── google
│   │   │   ├── android
│   │   │   │   └── updater-script
└── README.md
├── bin
│   └── ffmpeg
├── bin
│   └── zip
└── customize.sh
```


