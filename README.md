# JS Bundler
A Rust based toy bundler for Javascript
**For research purposes only, not production ready!**

Currently performs a "webpack" style bundling process that condenses all modules into a single file that provides a module dictionary that grants each module a `require` and `exports` object.

```Javascript
ex. 
const modules = {
	"modA": (require, exports) => {...},
	"modB": (require, exports) => {...},
}
```

## Handled Use Cases

 - [X] Transforms relative imports to `require` statements
 ```Javascript
 ex.
 import area from "../square" -> const { default: area} = require("../square");
 import area, { anotherFn } from "../square" -> const { default: area, anotherFn} = require("../square");
 ```
 - [X] Transforms exports to module exports
 ```Javascript
 export default area -> exports.default = area;
 export const area = _area -> exports.area = _area;
 export { area } -> exports.area = area;
 ```
- [ ] NodeJS modules imports
```Javascript
import express from "express"
```
- [ ] "Rollup" style bundling
- [ ] Bundle beautifier after generating
## To Run
```
cargo build --release
./target/release/bundler [entry point] [output directory]

# ex. ./target/release/bundler ./test/index.js ./test/out
```
