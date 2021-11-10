# TODO
- [ ] make the Plugins run after the node has been assembled.
    - [ ] Code fence processor plugin
    - [ ] tag processor.
- [ ] Use [rphtml](https://github.com/fefit/rphtml) for parsing the html
    as html5ever is not maintained anymore and causes some issue when compiled to the browser

# Issue
- A runtime error occurs when `ammonia`, `html5ever` and `markup5ever` is used and compiled into `wasm`
    - Uncaught TypeError: Error resolving module specifier “env”. Relative module specifiers must start with “./”, “../” or “/”.
