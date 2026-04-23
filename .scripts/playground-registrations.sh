if [ -z "$1" ]; then
    echo "No path to registration file was given";
    exit 1;
fi

for f in *; do
    if [ -d "$f" ]; then
        echo "Adding registration to '$f' policy example";
        cp $1 "$f/playground/config/registration.yaml"
    fi
done
