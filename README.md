<div align="center">

<img src="wechat-rs.ico" />

## 微信小工具

**一个完全由Rust实现的微信~~工具~~多开。**

</div>

---

## ✨ 缘由

> 都2021年了，为什么还写个多开的小工具？

多年前有个小工具为了快速实现，没有使用Rust开发，而是通过Golang实现注入和逻辑程序，C++实现的DLL。最近在用Rust重新实现，所以有必要进行测试和验证。

- 不会写🌚 C++
- Golang 不能做 inline Hook
- [Rust DLL注入工具](https://github.com/jiusanzhou/injrs)的预演
- Rust DLL 进行 Hook 的预演
- 有可能会有其他功能 🎉



## 👆 使用

1. 可以[下载已编译好的程序](https://github.com/jiusanzhou/multi-wechat-rs/releases)，或通过下面的步骤自行编译。

2. 双击运行程序会自动打开已安装的微信。

## 📦️ 编译

Rust环境准备。

Rust的可通过下面命令进行安装，
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
或查看 [https://rustup.rs/](https://rustup.rs/)


使用Cargo安装,
```bash
cargo install multi-wechat-rs
```

或代码Clone下来，并进入代码目录执行以下命令

```bash
cargo build --release
```


## 📺 心得

- FFI很方便，比Golang实现便捷
- Win32开发不熟悉，字符串处理等踩了坑
- DLL 和 Injector 全纯Rust完全可行
- 二进制大小满意，其中icon占据30kb


## ❤️ 鼓励

<img width="200" src="https://payone.wencai.app/s/zoe.png" alt="鼓励一下由 https://payone.wencai.app 赞助">

*鼓励一下由 https://payone.wencai.app 赞助*