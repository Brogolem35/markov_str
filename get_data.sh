#!/bin/bash

mkdir data

for i in {1..200}; do
  url="https://www.gutenberg.org/cache/epub/${i}/pg${i}.txt"
  output_file="data/pg${i}.txt"
  
  curl -s -o $output_file $url
  
  if [[ $? -eq 0 ]]; then
    echo "Downloaded pg${i}.txt"
    sed -i 's/“/"/g; s/”/"/g; s/–/-/g; s/…/.../g; s/‘/'\''/g; s/’/'\''/g' "$output_file"
  else
    echo "Failed to download pg${i}.txt"
  fi
done

