.PHONY: release run-release clean dist

release:
	cargo build --release

run-release:
	cargo run --release

dist: release
	@mkdir -p dist
	@cp -f "target/release/chatak" "dist/chatak"
	@printf '%s\n' '{' \
	'  "bookmarks": [],' \
	'  "last_dir": null,' \
	'  "openers": [' \
	'    { "name": "pdf", "extensions": ["pdf"], "command": "zathura", "args": ["{path}"] },' \
	'    { "name": "images", "extensions": ["png","jpg","jpeg","gif","webp","bmp","svg","ico"], "command": "feh", "args": ["{path}"] },' \
	'    { "name": "text", "extensions": ["txt","md","rs","toml","yaml","yml","json","js","ts","jsx","tsx","py","go","c","h","cpp","hpp","java","kt","kts","cs","html","css","scss","sql","sh","bash","zsh","fish","ini","conf","cfg","env"], "command": "nvim", "args": ["{path}"] }' \
	'  ]' \
	'}' > "dist/config.example.json"

clean:
	cargo clean
