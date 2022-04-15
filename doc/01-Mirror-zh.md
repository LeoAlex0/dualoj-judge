# \[可选\] 打包时镜像源的选择

> 如果有透明代理一类的玩意儿或者就是头铁的，可以忽略本节

本文档以如下镜像源为例, 如有需要，可自行魔改:
* tuna 的 [alpine 镜像源](https://mirrors.tuna.tsinghua.edu.cn/help/alpine/)
* [rsproxy 源](https://rsproxy.cn/)

以下配置以项目根目录为例，`examples/A+B Problem/judger` 的镜像配置与此同理

## 编译时 crate —— `.cargo/config` 文件

将如下内容写入 `.cargo/config`

```toml
[source.crates-io]
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
```

## 打包时镜像源

将如下内容写入 `script/setup-mirror.sh`， 本脚本将会在打包时的构建阶段运行

```bash
#!/bin/sh -xe

# alpine
sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories || echo 'not alpine'

# rustup
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
```
