# 公开资料行为基线

检索日期 2026-10-25。以下为官方指南的简述，不是本机截图或执行结果；不将指南内容自动等同于本机 17.00.17.02 的全部行为。

| 官方页面 | 可支持的行为 | 适用性/限制 |
|---|---|---|
| [Keys and parts](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/keys-and-parts) | 方向/滚动键、拨号、左右选择键及电源/结束键 | 该指南明确列 TA-1272、TA-1264、TA-1284、TA-1281、TA-1278、TA-1280；是 2020 产品指南 |
| [Keypad](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/keypad) | 主屏 Menu；四方向导航；按滚动键打开。文字输入按字母键后可用滚动键选择；0 空格；# 切换大小写；插入选项含符号 | 不能据此推断每种语言的默认输入模式、定时单位、长按阈值或中心键电气接线 |
| [Messages](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/messages?locale=en-US) | Menu→New message；填写号码或从联系人选择；编辑正文；插入表情/符号；Send | 作为顶层流程依据，未证明草稿/取消弹窗、短信分段及特定键在所有焦点位置的效果 |
| [Browse the web](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/browse-the-web) | 进入浏览器、输入地址、确认 | 官方明确浏览器因地区而不同且行为有差异；不能直接认定本机使用某一浏览器 |
| [Change the look](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/change-the-look-of-your-home-screen) | Settings→Personalisation→Lock screen background→Wallpapers→选择 | 证明指南描述的设置路径，不足以恢复全部主题状态和动态布局 |

公开框架参考来自 [ZW217_W19_V4.1_yisai 固定提交](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/tree/d48d55e028c13b6f50c291cbb7ea29201c3c8432)。保存的 35 个文本文件详见 provenance.json。它不是 TA-1278 官方源码，版权声明保留，未确认整个仓库的开源授权。

实际用途：将 native 事件编号与 caf.h/mmk_msg.h/mmi_appmsg.h 对照；按 window_parse.h 解释早期控件 opcode 和参数；按 T9 API 头解释语言编号和词库头。原始值、地址和控制流仍来自本机镜像。不同版本已存在 `END_WIN=f031` 与本机 `f032` 的差异，公开条件编译也未还原为本机 build 配置。所有符号关联标记为 framework-correlated，而非 recovered-source。

网络读取只下载公开文本，没有上传任何固件/设备数据，也没有编译或运行参考源码。未找到可代替本机证据的完整官方 TA-1278 源码或 UI 状态机。
