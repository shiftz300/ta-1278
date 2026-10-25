# Nokia TA-1278

[English](README.md)

**Nokia 215 4G（2020），TA-1278** 固件研究及可运行的桌面交互程序，固件版本 **17.00.17.02**。

## 运行

需要 Rust 和图形驱动。已在 macOS arm64 验证。

```sh
cargo run --locked -p phone-desktop
```

生成本地 macOS 应用：`sh scripts/package-macos.sh`，然后打开 `target/TA1278.app`。

- **短信：** 多击和直接输入、光标移动、换行、字符计数、草稿和本地待发箱。
- **浏览器：** HTTP(S) 文本网页、链接、输入框、复选框、选择框及 GET 表单。内置演示无需联网。
- **音乐：** 播放暂停、切歌、长按快进退、循环。进度为模拟，暂无音频输出。
- 电话、联系人暂未接入服务。**不会拨号或发送短信。**

方向键导航，Enter 确认，F1/F2 为软键，Escape 也可作右软键。鼠标**左/右/中键**分别触发**左软键/右软键/确认键**，不看点击位置。Home 返回主屏或提示保存短信。

编辑时，0–9 多击输入，长按输入数字；F4 切换 abc/ABC/123。也支持直接输入字母和桌面输入法。Tab 切换收件人与正文；Shift+Enter 换行；Backspace 删除；Cmd/Ctrl+A 全选。[操作与模块说明](docs/architecture.md)。

macOS 本地消息位于 `~/Library/Application Support/TA1278 Replica/`。可用 `--data-dir PATH` 指定测试目录。程序不会打开固件或手机个人文件。

## 研究状态

面向这个程序的必要调查已收尾。**原机整个系统及完整交互尚未全部恢复。** T9 候选、精确布局与时序、Dorado 内核和音频服务等仍未解。程序复用已提取的字体、图标和部分规则，其余设计选择均单独标注。

- [实现依据与边界](nokia-ta1278-research/analysis/ui/replica-readiness.md)
- [短信、浏览器、音乐规则](nokia-ta1278-research/analysis/ui/apps/interaction-core.md)
- [尚未恢复的部分](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [固件结构](nokia-ta1278-research/analysis/firmware-structure.md)、[来源](nokia-ta1278-research/sources.md)、[复现步骤](PUBLICATION.md)
- [验证记录](docs/validation.md)

系统固件及派生素材保留在 `nokia-ta1278-research/`；私人 NAND/NV 和采集临时文件不提交。见[固件清单](nokia-ta1278-research/firmware/public-system-manifest.json)及[第三方声明](THIRD_PARTY_NOTICES.md)。
