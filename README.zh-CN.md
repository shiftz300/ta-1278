# Nokia TA-1278

[English](README.md)

**Nokia 215 4G（2020），TA-1278** 固件研究及可运行的桌面交互程序，固件版本 **17.00.17.02**。

## 运行

需要 Rust 和图形驱动。已在 macOS arm64 验证。

```sh
cargo run --release --locked -p phone-desktop
```

生成优化版 macOS 应用：`sh scripts/package-macos.sh`，然后打开 `target/TA1278.app`。

- **短信：** 拼音、笔画、Abc/abc/ABC 多击和直接输入、光标移动、换行、字符计数、草稿、会话列表、收发气泡、底部续写、转发、确认删除和本地待发记录。
- **浏览器：** HTTP(S) 文本网页、链接、输入框、单选/复选框、选择框及 GET 表单；书签、历史和离线页面可保存。内置演示无需联网。[浏览器功能与边界](docs/browser-capabilities.md)。
- **音乐：** 从虚拟手机存储和存储卡真实播放 MP3/WAV，支持暂停、切歌、长按快进退、音量、循环、随机播放和持久化播放列表。联系人图片、壁纸和铃声共用文件选择器；可从“文件”打开本地图片和视频。
- 日历、闹钟、本地联系人、计算器、定时器、秒表和使用原机地图与素材的贪吃蛇已可操作。电话及通话记录入口置灰并禁用。**桌面程序不会拨号或发送真实短信。**

方向键导航，Enter 确认，F1/F2 为软键，Escape 也可作右软键。鼠标**左/右/中键**分别触发**左软键/右软键/确认键**，不看点击位置。Home 返回主屏或提示保存短信。

编辑时，0–9 按当前模式输入，长按输入数字；F4 切换 拼 → 笔 → Abc → abc → ABC → 123。候选框中，上下选词，左右切换拼音与文字列；确认键选词。也支持直接输入字母和桌面输入法。Tab 切换收件人与正文；Shift+Enter 换行；Backspace 删除；Cmd/Ctrl+A 全选。[操作与模块说明](docs/architecture.md)。

macOS 本地消息、设置、浏览记录及日历/联系人数据位于 `~/Library/Application Support/TA1278 Replica/`。可用 `--data-dir PATH` 指定测试目录。计时器、秒表分段和待处理提醒也会保留到下次启动。[保存与恢复](docs/persistence.md)。程序不打开固件和采集存档；媒体只从配置的 VFS 卷读取。

存储使用程序内部的 VFS，不设导入按钮、不安装系统驱动。默认将数据目录中的 `files/` 和 `card/` 挂到 `/phone` 和 `/card`。把音乐放进去，再在“音乐 → 菜单 → 手动刷新”更新曲库。“文件”应用浏览相同的卷。

```sh
cargo run --release --locked -p phone-desktop -- --mount-card /你的/存储卡目录
# 可选：--read-only-storage --disable network,music,files,pictures,video,sounds
# 不挂存储卡：--no-card
```

`--mount-card` 直接使用指定目录，手机内的文件操作会影响该目录。只读开关只限制媒体卷写入，不影响短信和设置保存。[存储、开关与边界](docs/storage.md)。

[贪吃蛇代码依据](docs/snake-recovery.md) · [闹钟代码证据](docs/alarm-recovery.md) · [短信读写流程](docs/sms-lifecycle.md) · [本地收件测试](docs/sms-inbox.md) · [体积与优化检查](docs/build-size.md)

[交付范围](docs/remaining-work.md) · [本轮补齐的流程和明确边界](docs/local-completion.md)

目标是独立、注重还原的手机交互组件。交付不再以完整解读固件为前提。

## 研究状态

**逆向仍未完成，当前程序也尚未通过原机逐项比对。** 背景已按用户实机反馈改为纯黑，接入六组固件主题色、三列图标和浮动候选，并修正短信菜单及保存退出流程。预测算法按要求使用替代实现，原机字体优先、Noto 补缺字。新增实机采样后，浏览器和短信排版已继续修正，共有 172 张公开参考截图。见[浏览器与短信对照](docs/browser-layout.md)、[本轮证据与改动](docs/native-ui.md)和[剩余差距](docs/fidelity.md)。

- [实现依据与边界](nokia-ta1278-research/analysis/ui/replica-readiness.md)
- [短信、浏览器、音乐规则](nokia-ta1278-research/analysis/ui/apps/interaction-core.md)
- [尚未恢复的部分](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [固件结构](nokia-ta1278-research/analysis/firmware-structure.md)、[来源](nokia-ta1278-research/sources.md)、[复现步骤](PUBLICATION.md)
- [实机自动采样脚本](docs/handset-automation.md)
- [实机截屏调查](docs/handset-capture.md)
- [验证记录](docs/validation.md)

系统固件及派生素材保留在 `nokia-ta1278-research/`；私人 NAND/NV 和采集临时文件不提交。见[固件清单](nokia-ta1278-research/firmware/public-system-manifest.json)及[第三方声明](THIRD_PARTY_NOTICES.md)。

- [实机截图与观察记录](docs/handset-observations.md)：已打通截图和按键，完整采样与复刻对比仍在进行。

原机 MIDI 和提取出的音色库现已用于铃声与贪吃蛇音效的本地合成。[证据与边界](docs/native-audio.md)。

[全量成果核对与实现对应](docs/reverse-audit.md) · [当前包体组成](docs/build-size.md)
