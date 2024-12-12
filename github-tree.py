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
    # Create a nested structure and count files and folders
    structure = {}
    file_count = 0
    folder_count = 0

    for item in tree:
        parts = item["path"].split("/")
        current_level = structure
        for part in parts:
            if part not in current_level:
                current_level[part] = {}
            current_level = current_level[part]

        # Count files and folders
        if item["type"] == "blob":  # 'blob' indicates a file
            file_count += 1
        elif item["type"] == "tree":  # 'tree' indicates a folder
            folder_count += 1

    def print_nested(d, prefix=""):
        for i, (key, value) in enumerate(d.items()):
            connector = "└── " if i == len(d) - 1 else "├── "
            print(prefix + connector + key)
            print_nested(value, prefix + ("    " if connector == "└── " else "│   "))

    # Print the directory structure
    print_nested(structure)

    # Print the counts
    print("\nTotal folders:", folder_count)
    print("Total files:", file_count)

def main():
    if len(sys.argv) != 2:
        print("Usage: python github-tree.py <GitHub Repository URL>")
        return
    
    repo_url = sys.argv[1]
    fetch_tree(repo_url)

if __name__ == "__main__":
    main()
