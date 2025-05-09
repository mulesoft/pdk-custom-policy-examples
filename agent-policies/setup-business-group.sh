#!/bin/bash

# Parse the JSON and extract id and name
json=`anypoint-cli-v4 account business-group list -o=json`

# Extract ids and names using jq
ids=($(echo "$json" | jq -r '.[] | .id'))
names=($(echo "$json" | jq -r '.[] | .name'))

# Display options to the user
echo "Please select an organization:"
for i in "${!names[@]}"; do
  echo "$i) ${names[$i]}"
done

# Read user selection
read -p "Enter the number corresponding to your choice: " choice

# Get the selected id
selected_id=${ids[$choice]}

# Find all Cargo.toml files under directories that end with "-policy"
cargo_toml_files=$(find . -type f -path "*-policy/Cargo.toml")

if [ -z "$cargo_toml_files" ]; then
  echo "No Cargo.toml files found in -policy directories!"
  exit 1
fi

# Regular expression for matching UUIDs
UUID_REGEX='[0-9a-fA-F]\{8\}-[0-9a-fA-F]\{4\}-[0-9a-fA-F]\{4\}-[0-9a-fA-F]\{4\}-[0-9a-fA-F]\{12\}'


# Loop through each Cargo.toml file found and update it
for cargo_toml in $cargo_toml_files; do
  cargo_wc="${cargo_toml}.tmp"
  cp "$cargo_toml" "${cargo_wc}"  # Backup the original file
  cat ${cargo_wc} | sed "s/\"{group_id_value}\"/\"$selected_id\"/" > $cargo_toml
  rm ${cargo_wc}
  echo "Updated $cargo_toml with id: $selected_id"
done