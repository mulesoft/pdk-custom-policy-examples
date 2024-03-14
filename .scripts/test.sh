if [ -z "$1" ]; then
    echo "No path to registration file was given";
    exit 1;
fi

for f in *; do
    if [ -d "$f" ]; then
        echo "Testing '$f' policy example";
        cd "$f";
        cp $1 ./tests/common/registration.yaml
        make setup && make test;
        rm ./tests/common/registration.yaml
        cd ..;
    fi
done
