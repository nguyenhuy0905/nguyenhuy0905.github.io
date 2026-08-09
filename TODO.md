# To-do
## SSG
- [x] Scan for blocks.
- [ ] Add some arithmetic operations.
- [ ] Parse:
    - [ ] Expressions.
    - [ ] Statements.
- [ ] Add more diagnostics
    - It's probably better to keep the diagnostic infos in a separate vector. I'll rarely access the diagnostics, but if I need to access that info, I'll access the entire package at once.

## Other things
- Change the include mechanism. I should make includes be an unary expression, and the thing unary-ed on should be a string, and that's all good.
