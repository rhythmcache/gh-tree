import os
import requests
import sys
import re
def download_file(file_url, output_path):
    print(f"Downloading {file_url}...")
    response = requests.get(file_url)
    response.raise_for_status()
    with open(output_path, 'wb') as f:
        f.write(response.content)
    print(f"Saved {file_url} to {output_path}")
def download_directory(repo_url, dir_path, output_dir):
    print(f"Downloading directory: {dir_path}...")
    api_url = f"https://api.github.com/repos/{repo_url[19:]}/contents/{dir_path}"
    response = requests.get(api_url)
    response.raise_for_status() 
    content = response.json()
    for item in content:
        item_name = item['name']
        item_path = os.path.join(dir_path, item_name)
        if item['type'] == 'file':
            raw_file_url = item['download_url']
            download_file(raw_file_url, os.path.join(output_dir, item_name))
        elif item['type'] == 'dir':
            new_output_dir = os.path.join(output_dir, item_name)
            os.makedirs(new_output_dir, exist_ok=True)
            download_directory(repo_url, item_path, new_output_dir)
def pull_github(url):
    if not url.startswith('https://github.com/'):
        print("Error: The URL must be a GitHub URL.")
        return
    match = re.match(r'https://github.com/([^/]+)/([^/]+)/(?:blob|tree)/(main|master)/(.*)', url)
    if not match:
        print("Error: Invalid URL format.")
        return
    user_repo = match.group(1) + "/" + match.group(2)
    repo_url = f"https://github.com/{user_repo}"
    repo_path = match.group(3)
    if '/blob/' in url:
        raw_file_url = f"https://raw.githubusercontent.com/{user_repo}/master/{repo_path}"
        output_path = os.path.basename(repo_path)
        download_file(raw_file_url, output_path)
    elif '/tree/' in url:
        output_dir = os.path.basename(repo_path)
        os.makedirs(output_dir, exist_ok=True)
        download_directory(repo_url, repo_path, output_dir)
if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python github_pull.py <GitHub URL>")
    else:
        pull_github(sys.argv[1])