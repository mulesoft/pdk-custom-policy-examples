for f in *; do
    if [ -d "$f" ]; then
        echo "Building '$f' policy example";
        cd "$f" && make setup && make build && cd ..;
    fi
done
