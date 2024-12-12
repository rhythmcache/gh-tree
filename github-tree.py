import requests
import sys
from urllib.parse import urlparse

def fetch_tree(repo_url):
    if repo_url.endswith(".git"):
        repo_url = repo_url[:-4]
    
    parsed_url = urlparse(repo_url)
    if not parsed_url.netloc.endswith("github.com"):
        print("Error: Invalid GitHub URL.")
        return
    
    path_parts = parsed_url.path.strip("/").split("/")
    if len(path_parts) < 2:
        print("Error: Unable to parse user and repository from URL.")
        return
    
    user, repo = path_parts[:2]
    branch = "main"
    if "tree" in path_parts:
        branch = path_parts[path_parts.index("tree") + 1]
    
    api_url = f"https://api.github.com/repos/{user}/{repo}/git/trees/{branch}?recursive=1"
    
    response = requests.get(api_url)
    if response.status_code == 200:
        tree = response.json()["tree"]
        print_tree(tree)
    else:
        print(f"Error: Unable to fetch repository tree (status code {response.status_code}).")
        print(response.json().get("message", "No additional information provided."))

def print_tree(tree):
    paths = [item["path"] for item in tree]
    paths.sort()

    current_prefix = ""
    for path in paths:
        parts = path.split("/")
        for depth, part in enumerate(parts):
            prefix = "│   " * depth + "├── " if depth < len(parts) - 1 else "│   " * depth + "└── "
            if prefix.strip() != current_prefix:
                print(prefix + part)
                current_prefix = prefix.strip()

def main():
    if len(sys.argv) != 2:
        print("Usage: python github-tree.py <GitHub Repository URL>")
        return
    
    repo_url = sys.argv[1]
    fetch_tree(repo_url)

if __name__ == "__main__":
    main()
