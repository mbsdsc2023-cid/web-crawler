sudo cross build --target x86_64-pc-windows-gnu --release
mkdir -p ./web-crawler
cp -r -p ./public ./web-crawler/
cp -p ./target/x86_64-pc-windows-gnu/release/web-crawler.exe ./web-crawler/