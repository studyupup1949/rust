src/lib.rs: syntax.ebnf ../node_modules/.bin/syntax-cli
	../node_modules/.bin/syntax-cli -g $< -m LALR1 -o $@

../node_modules/.bin/syntax-cli:
	yarn --pure-lockfile