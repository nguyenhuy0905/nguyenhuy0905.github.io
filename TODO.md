# To-do
## SSG
- [x] Scan for blocks.
- [ ] Add some arithmetic operations.
- [ ] Parse:
    - [ ] Expressions.
    - [ ] Statements.
- [ ] Add more diagnostics
    - It's probably better to keep the diagnostic infos in a separate vector. I'll rarely access the diagnostics, but if I need to access that info, I'll access the entire package at once.
    - Update 2026-08-24: The way the positions are added to the positions vector is quite, bad. If a token has more than 1 character, that token's position is the position of its last character. So, we'll probably need to save a `curr_pos` inside the `Lex` struct.

## Other things
- Change the include mechanism. I should make includes be an unary expression, and the thing unary-ed on should be a string, and that's all good.
