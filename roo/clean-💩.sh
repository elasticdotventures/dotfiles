#!/bin/bash

# Get current user's home directory
USER_HOME=$(eval echo ~"$USER")
TASKS_DIR="$USER_HOME/.vscode-server/data/User/globalStorage/rooveterinaryinc.roo-cline/tasks"

# Safety check: does the directory exist?
if [ ! -d "$TASKS_DIR" ]; then
  echo "Directory $TASKS_DIR does not exist."
  exit 1
fi

echo "Scanning for subdirectories over 1 GB in $TASKS_DIR..."

# Find and delete subdirectories >1 GB
find "$TASKS_DIR" -mindepth 1 -maxdepth 1 -type d -exec du -s --block-size=1G {} + \
  | awk '$1 > 1 {print $2}' \
  | while read -r dir; do
      echo "Deleting $dir..."
      rm -rf "$dir"
  done

echo "Cleanup complete."

