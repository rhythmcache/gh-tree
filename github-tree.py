# github-tree.py
# by coldnw.t.me
# github.com/rhythmcache

import requests
import sys
import os
from urllib.parse import urlparse

# Add your GitHub API token here if available
API_TOKEN = ""  # Replace with "TOKEN-VALUE" if needed

def fetch_repo_info(user, repo):
    url = f"https://api.github.com/repos/{user}/{repo}"
    headers = get_headers()
    response = requests.get(url, headers=headers)
    if response.status_code == 200:
        return response.json()
    else:
        print("Error fetching repository info.")
        return None

def fetch_tree_data(user, repo, branch):
    url = f"https://api.github.com/repos/{user}/{repo}/git/trees/{branch}?recursive=1"
    headers = get_headers()
    all_items = []

    while url:
        response = requests.get(url, headers=headers)
        if response.status_code == 200:
            data = response.json()
            all_items.extend(data["tree"])
            url = response.links.get('next', {}).get('url')  # Pagination
        elif response.status_code == 403:
            print("Rate limit exceeded. Try again later.")
            return None
        else:
            print(f"Error fetching tree: {response.status_code}")
            print(response.json().get("message", "No additional information provided."))
            return None

    return all_items

def parse_github_url(repo_url):
    if repo_url.endswith(".git"):
        repo_url = repo_url[:-4]
    
    parsed_url = urlparse(repo_url)
    if not parsed_url.netloc.endswith("github.com"):
        return None, None

    path_parts = parsed_url.path.strip("/").split("/")
    if len(path_parts) < 2:
        return None, None

    return path_parts[:2]

def create_placeholder_structure(tree, base_path):
    for item in tree:
        path = os.path.join(base_path, item["path"])
        if item["type"] == "tree":
            os.makedirs(path, exist_ok=True)
        elif item["type"] == "blob":
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w") as f:
                pass

    print(f"Placeholder structure created at: {base_path}")

def print_tree(tree):
    structure = {}

    for item in tree:
        parts = item["path"].split("/")
        current_level = structure
        for part in parts:
            current_level = current_level.setdefault(part, {})

    def print_nested(d, prefix=""):
        for i, (key, value) in enumerate(d.items()):
            connector = "└── " if i == len(d) - 1 else "├── "
            print(prefix + connector + key)
            print_nested(value, prefix + ("    " if connector == "└── " else "│   "))

    print_nested(structure)

def count_files_and_folders(tree):
    files = sum(1 for item in tree if item["type"] == "blob")
    folders = sum(1 for item in tree if item["type"] == "tree")
    return files, folders

def get_headers():
    """Prepare headers for the request, including API token if available."""
    headers = {}
    if API_TOKEN:
        print("Using API token to fetch data...")
        headers["Authorization"] = f"token {API_TOKEN}"
    return headers

def main():
    if len(sys.argv) < 3:
        print("Usage:")
        print(f"  python {os.path.basename(__file__)} touch <output-path> <GitHub Repository URL> [branch]")
        print(f"  python {os.path.basename(__file__)} view <GitHub Repository URL> [branch]")
        return

    command = sys.argv[1]
    if command not in ["touch", "view"]:
        print(f"Error: Unsupported command '{command}'.")
        return

    if command == "touch":
        if len(sys.argv) < 4:
            print("Error: Missing arguments for 'touch' command.")
            return
        
        base_path = sys.argv[2]
        repo_url = sys.argv[3]
        branch = sys.argv[4] if len(sys.argv) > 4 else None
        user, repo = parse_github_url(repo_url)

        if not user or not repo:
            print("Error: Invalid GitHub URL.")
            return

        if not branch:
            repo_info = fetch_repo_info(user, repo)
            branch = repo_info.get("default_branch", "main") if repo_info else "main"

        tree = fetch_tree_data(user, repo, branch)
        if tree:
            create_placeholder_structure(tree, base_path)

    elif command == "view":
        if len(sys.argv) < 3:
            print("Error: Missing arguments for 'view' command.")
            return
        
        repo_url = sys.argv[2]
        branch = sys.argv[3] if len(sys.argv) > 3 else None
        user, repo = parse_github_url(repo_url)

        if not user or not repo:
            print("Error: Invalid GitHub URL.")
            return

        if not branch:
            repo_info = fetch_repo_info(user, repo)
            branch = repo_info.get("default_branch", "main") if repo_info else "main"

        tree = fetch_tree_data(user, repo, branch)
        if tree:
            print_tree(tree)
            files, folders = count_files_and_folders(tree)
            print(f"\nTotal folders: {folders}")
            print(f"Total files: {files}")

if __name__ == "__main__":
    main()
    
