# term-kit - History Finder

一个基于Rust构建的终端工具集。首个工具「History Finder」帮助您快速浏览、搜索和复用终端历史命令。

## ✨ 功能特性

- **终端历史导航**
  - 展示最多1000条历史命令（按时间倒序）
  - 支持上下键选择、快捷键操作
- **即时搜索**
  - 输入 `/` 开头内容进行实时过滤
  - 高亮匹配结果
- **快速交互**
  - 一键复制选中命令到剪贴板
  - 直观的三窗格TUI界面
- **跨平台支持**
  - 支持Linux/macOS/Windows终端

## 🛠️ 安装

### 前置需求
- Rust工具链 (1.65+)
- 系统剪贴板支持 (xclip/macOS pbcopy/Win32 API)

### 安装步骤
```bash
$ git clone https://github.com/WilsonHuang080705/term-kit.git
$ cd term-kit
$ cargo run --bin main
```

## 🕹️ 使用指南

### 界面布局
```
+-----------------------------------+
| term-kit v0.1.0 | ↑/↓: Navigate   |
|                 | Enter: Copy     |
+-----------------------------------+
| 1. git commit -m "initial commit" |
| 2. cargo build --release          |
| 3. ssh user@example.com           |
| ... (1000 entries max)            |
+-----------------------------------+
| > /build      | i: Input  q: Quit |
+-----------------------------------+
```

### 快捷键
| 按键    | 功能                         |
|---------|------------------------------|
| ↑/↓     | 上下移动选择                 |
| Enter   | 复制选中命令到剪贴板         |
| i       | 进入输入模式                 |
| /       | 开始搜索（输入模式中自动添加）|
| Esc     | 取消输入/返回导航模式        |
| h       | 显示帮助信息                 |
| q       | 退出程序                     |

## 📦 依赖项
- [crossterm](https://crates.io/crates/crossterm) - 跨平台终端控制
- [ratatui](https://crates.io/crates/ratatui) - 终端用户界面构建
- [copypasta](https://crates.io/crates/copypasta) - 剪贴板操作
- [serde](https://crates.io/crates/serde) - 序列化
- [directories](https://crates.io/crates/directories) - 获取历史文件路径
- [textwrap](https://crates.io/crates/textwrap/) - 文本包裹
- [wl-clipboard-rs](https://crates.io/crates/wl-clipboard-rs) - 跨平台剪贴板操作

## 🤝 贡献
欢迎提交Issue和PR！请遵循以下步骤：
1. Fork项目仓库
2. 创建特性分支 (`git checkout -b feature/awesome`)
3. 提交修改 (`git commit -am 'Add awesome feature'`)
4. 推送分支 (`git push origin feature/awesome`)
5. 创建Pull Request

## 📄 许可证
该项目许可证由Apache提供支持，详情可见[LICENSE](LICENSE)
