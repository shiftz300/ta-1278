# Nokia TA-1278

[English](README.md)

**Nokia 215 4G（2020），TA-1278** 固件研究及可运行的桌面交互程序，固件版本 **17.00.17.02**。

## 运行

需要 Rust 和图形驱动。已在 macOS arm64 验证。

```sh
cargo run --locked -p phone-desktop
```

生成本地 macOS 应用：`sh scripts/package-macos.sh`，然后打开 `target/TA1278.app`。

- **短信：** 拼音、笔画、Abc/abc/ABC 多击和直接输入、光标移动、换行、字符计数、草稿和本地待发箱。
- **浏览器：** HTTP(S) 文本网页、链接、输入框、单选/复选框、选择框及 GET 表单；书签、历史和离线页面可保存。内置演示无需联网。
- **音乐：** 播放暂停、切歌、长按快进退、循环。进度为模拟，暂无音频输出。
- 日历、闹钟、本地联系人、计算器、定时器、秒表和替代版贪吃蛇已可操作。电话未接入服务。**不会拨号或发送短信。**

方向键导航，Enter 确认，F1/F2 为软键，Escape 也可作右软键。鼠标**左/右/中键**分别触发**左软键/右软键/确认键**，不看点击位置。Home 返回主屏或提示保存短信。

编辑时，0–9 按当前模式输入，长按输入数字；F4 切换 拼 → 笔 → Abc → abc → ABC → 123。候选框中，上下选词，左右切换拼音与文字列；确认键选词。也支持直接输入字母和桌面输入法。Tab 切换收件人与正文；Shift+Enter 换行；Backspace 删除；Cmd/Ctrl+A 全选。[操作与模块说明](docs/architecture.md)。

macOS 本地消息、设置、浏览记录及日历/联系人数据位于 `~/Library/Application Support/TA1278 Replica/`。可用 `--data-dir PATH` 指定测试目录。程序不会打开固件或手机个人文件。

## 研究状态

**逆向仍未完成，当前程序也尚未通过原机逐项比对。** 背景已按用户实机反馈改为纯黑，接入六组固件主题色、三列图标和浮动候选，并修正短信菜单及保存退出流程。预测算法按要求使用替代实现，原机字体优先、Noto 补缺字。新增实机采样后，浏览器和短信排版已继续修正，共有 160 张公开参考截图。见[浏览器与短信对照](docs/browser-layout.md)、[本轮证据与改动](docs/native-ui.md)和[剩余差距](docs/fidelity.md)。

- [实现依据与边界](nokia-ta1278-research/analysis/ui/replica-readiness.md)
- [短信、浏览器、音乐规则](nokia-ta1278-research/analysis/ui/apps/interaction-core.md)
- [尚未恢复的部分](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [固件结构](nokia-ta1278-research/analysis/firmware-structure.md)、[来源](nokia-ta1278-research/sources.md)、[复现步骤](PUBLICATION.md)
- [实机自动采样脚本](docs/handset-automation.md)
- [实机截屏调查](docs/handset-capture.md)
- [验证记录](docs/validation.md)

系统固件及派生素材保留在 `nokia-ta1278-research/`；私人 NAND/NV 和采集临时文件不提交。见[固件清单](nokia-ta1278-research/firmware/public-system-manifest.json)及[第三方声明](THIRD_PARTY_NOTICES.md)。

- [实机截图与观察记录](docs/handset-observations.md)：已打通截图和按键，完整采样与复刻对比仍在进行。
