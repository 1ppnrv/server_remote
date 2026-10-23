app=remote
host=mrbap@192.168.1.62

cd ~/dev/serveur/$app
./test.sh

cargo build --release

cp ./target/release/$app ./release
cp -r ./data ./release


rsync -z --chmod=755 "target/release/$app" "$host:/opt/$app/$app.new"
rsync -az --chmod=755 "data/" "$host:/opt/$app/data"

ssh server "mv /opt/$app/$app.new /opt/$app/$app && sudo systemctl restart $app"