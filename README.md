First, tag a commit before pushing it to kick of release CI

```bash
# after commiting
git tag v0.1.0
# check if the the ag was applied to latest commit
git log --oneline
# then push (it doesn't push tags by default)
git push && git push --tags
```

## references

- A lot of stuff learnt from [shadowsocks source code](https://github.com/shadowsocks/shadowsocks-rust)
- [how to add git tags](https://devconnected.com/how-to-create-git-tags/)
- [compile for osx](https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html)
