package main

import (
    "encoding/json"
    "flag"
    "fmt"
    "io/ioutil"
    "net/http"
    "os"
    "path/filepath"
    "strings"
)

type TreeResponse struct {
    Tree []TreeItem `json:"tree"`
}

type TreeItem struct {
    Path string `json:"path"`
    Type string `json:"type"`
}

type RepoInfo struct {
    DefaultBranch string `json:"default_branch"`
}

func parseGitHubURL(repoURL string) (string, string) {
    repoURL = strings.TrimSuffix(repoURL, ".git")
    parts := strings.Split(repoURL, "/")
    if len(parts) < 2 {
        return "", ""
    }
    return parts[len(parts)-2], parts[len(parts)-1]
}

func fetchRepoInfo(user, repo, pat string) (*RepoInfo, error) {
    url := fmt.Sprintf("https://api.github.com/repos/%s/%s", user, repo)
    client := &http.Client{}
    req, err := http.NewRequest("GET", url, nil)
    if err != nil {
        return nil, err
    }

    if pat != "" {
        req.Header.Add("Authorization", "token "+pat)
    }
    
    resp, err := client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != 200 {
        return nil, fmt.Errorf("API error: %d", resp.StatusCode)
    }

    var info RepoInfo
    if err := json.NewDecoder(resp.Body).Decode(&info); err != nil {
        return nil, err
    }
    return &info, nil
}

func fetchTreeData(user, repo, branch, pat string) ([]TreeItem, error) {
    url := fmt.Sprintf("https://api.github.com/repos/%s/%s/git/trees/%s?recursive=1", user, repo, branch)
    client := &http.Client{}
    req, err := http.NewRequest("GET", url, nil)
    if err != nil {
        return nil, err
    }

    if pat != "" {
        req.Header.Add("Authorization", "token "+pat)
    }

    resp, err := client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    if resp.StatusCode != 200 {
        return nil, fmt.Errorf("API error: %d", resp.StatusCode)
    }

    var treeResp TreeResponse
    if err := json.NewDecoder(resp.Body).Decode(&treeResp); err != nil {
        return nil, err
    }
    return treeResp.Tree, nil
}

func createPlaceholderStructure(tree []TreeItem, basePath string) error {
    for _, item := range tree {
        path := filepath.Join(basePath, item.Path)
        if item.Type == "tree" {
            if err := os.MkdirAll(path, 0755); err != nil {
                return err
            }
        } else if item.Type == "blob" {
            dir := filepath.Dir(path)
            if err := os.MkdirAll(dir, 0755); err != nil {
                return err
            }
            if err := ioutil.WriteFile(path, []byte{}, 0644); err != nil {
                return err
            }
        }
    }
    return nil
}

type Node struct {
    Name     string
    Children map[string]*Node
}

func printTree(tree []TreeItem) {
    root := &Node{Children: make(map[string]*Node)}
    
    // Build tree structure
    for _, item := range tree {
        parts := strings.Split(item.Path, "/")
        current := root
        for _, part := range parts {
            if current.Children[part] == nil {
                current.Children[part] = &Node{
                    Name:     part,
                    Children: make(map[string]*Node),
                }
            }
            current = current.Children[part]
        }
    }

    // Print tree structure
    var printNode func(*Node, string, bool)
    printNode = func(node *Node, prefix string, isLast bool) {
        if node.Name != "" {
            connector := "├── "
            if isLast {
                connector = "└── "
            }
            fmt.Println(prefix + connector + node.Name)
        }

        keys := make([]string, 0, len(node.Children))
        for k := range node.Children {
            keys = append(keys, k)
        }

        for i, key := range keys {
            newPrefix := prefix
            if node.Name != "" {
                if isLast {
                    newPrefix += "    "
                } else {
                    newPrefix += "│   "
                }
            }
            printNode(node.Children[key], newPrefix, i == len(keys)-1)
        }
    }

    printNode(root, "", true)
}

func countFilesAndFolders(tree []TreeItem) (int, int) {
    files, folders := 0, 0
    for _, item := range tree {
        if item.Type == "blob" {
            files++
        } else if item.Type == "tree" {
            folders++
        }
    }
    return files, folders
}

func main() {
    touchCmd := flag.NewFlagSet("touch", flag.ExitOnError)
    touchPath := touchCmd.String("path", "", "Output path for the placeholder structure")
    touchPat := touchCmd.String("pat", "", "GitHub Personal Access Token")

    viewCmd := flag.NewFlagSet("view", flag.ExitOnError)
    viewPat := viewCmd.String("pat", "", "GitHub Personal Access Token")

    if len(os.Args) < 2 {
        fmt.Println("Expected 'touch' or 'view' subcommands")
        os.Exit(1)
    }

    switch os.Args[1] {
    case "touch":
        touchCmd.Parse(os.Args[2:])
        args := touchCmd.Args()
        if len(args) < 1 || *touchPath == "" {
            fmt.Println("Usage: github-tree touch -path=<output-path> -pat=<token> <repo-url> [branch]")
            os.Exit(1)
        }

        repoURL := args[0]
        branch := ""
        if len(args) > 1 {
            branch = args[1]
        }

        user, repo := parseGitHubURL(repoURL)
        if user == "" || repo == "" {
            fmt.Println("Invalid GitHub URL")
            os.Exit(1)
        }

        if branch == "" {
            info, err := fetchRepoInfo(user, repo, *touchPat)
            if err != nil {
                fmt.Printf("Error fetching repo info: %v\n", err)
                os.Exit(1)
            }
            branch = info.DefaultBranch
        }

        tree, err := fetchTreeData(user, repo, branch, *touchPat)
        if err != nil {
            fmt.Printf("Error fetching tree data: %v\n", err)
            os.Exit(1)
        }

        if err := createPlaceholderStructure(tree, *touchPath); err != nil {
            fmt.Printf("Error creating structure: %v\n", err)
            os.Exit(1)
        }
        fmt.Printf("Placeholder structure created at: %s\n", *touchPath)

    case "view":
        viewCmd.Parse(os.Args[2:])
        args := viewCmd.Args()
        if len(args) < 1 {
            fmt.Println("Usage: github-tree view -pat=<token> <repo-url> [branch]")
            os.Exit(1)
        }

        repoURL := args[0]
        branch := ""
        if len(args) > 1 {
            branch = args[1]
        }

        user, repo := parseGitHubURL(repoURL)
        if user == "" || repo == "" {
            fmt.Println("Invalid GitHub URL")
            os.Exit(1)
        }

        if branch == "" {
            info, err := fetchRepoInfo(user, repo, *viewPat)
            if err != nil {
                fmt.Printf("Error fetching repo info: %v\n", err)
                os.Exit(1)
            }
            branch = info.DefaultBranch
        }

        tree, err := fetchTreeData(user, repo, branch, *viewPat)
        if err != nil {
            fmt.Printf("Error fetching tree data: %v\n", err)
            os.Exit(1)
        }

        printTree(tree)
        files, folders := countFilesAndFolders(tree)
        fmt.Printf("\nTotal folders: %d\n", folders)
        fmt.Printf("Total files: %d\n", files)

    default:
        fmt.Printf("Unknown command: %s\n", os.Args[1])
        os.Exit(1)
    }
}
