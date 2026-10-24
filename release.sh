app=remote
host=server

cd ~/dev/serveur/$app
./test.sh

cargo build --release

cp ./target/release/$app ./release
cp -r ./data ./release

echo "sync executable"
rsync -z --chmod=755 "target/release/$app" "$host:/opt/$app/$app.new"
echo "sync data"
rsync -az --chmod=755 "data/" "$host:/opt/$app/data"

echo "begin ssh commands"
ssh server "mv /opt/$app/$app.new /opt/$app/$app && sudo systemctl restart $app"