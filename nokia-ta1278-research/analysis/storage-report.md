# 实机存储模式调查

2026-10-25 05:20–05:23，Asia/Shanghai。用户已备份并明确撤销先前读写限制。此次实际操作仍以查询元数据为主，没有格式化、刷写或复制整盘。

## USB 与磁盘证据

- 被动监听于 05:20:08 首次捕获设备；快照 analysis/environment/20261025T052019+0800.json 保存驱动和接口信息。
- USB 设备：2e04:0022；Spreadtrum / Generic Disk1.0；bcdUSB=0x0200，bcdDevice=0x0001；系统报告 480 Mb/s。
- 当前配置 1，接口 0；类 0x08（Mass Storage）、子类 0x06（SCSI transparent）、协议 0x50（Bulk-Only），端点数 2；此快照未取得端点地址。
- 驱动链包括 AppleUSBHostCompositeDevice、IOUSBMassStorageDriver、SCSI block device、IOMedia。当前仅观察到该存储接口，没有诊断/串口/MTP 接口。
- 沙箱内 libusb 仍返回 0；沙箱外同一程序返回 1，ID 与注册表一致。因此不能再单凭沙箱内 libusb 的空列表作设备不存在判断。
- diskutil 在沙箱内报 DiskManagement framework 不可用；沙箱外成功。
- /dev/disk4：外接物理 USB，LOCAL DISK，32,751,616 bytes，设备块大小 1024，可写、可移除，挂载点 /Volumes/NO NAME，MS-DOS FAT12，diskutil list 未列子分区。
- 这些结果证明正常数据链路工作，不证明启动协议或 FDL 兼容性，也不证明能访问整颗 128 MB 标称存储。

## 可见文件元数据

排除 .Trashes、.Spotlight-V100、.fseventsd、.TemporaryItems 且不跟随符号链接后，目录计数 34、文件 365，总文件大小 20,451,364 bytes。没有 .pac/.spd/.img/.bin/.elf/.axf/.fw/.upd 扩展名的候选。仅按文件名筛选，不能证明不存在改名固件。

可见目录包括 Photos、Recordings、Music files、Video、System、browser、PB、xysdk、Filearray。System 下有 MMS、Gallery、moreringset、Mp3_res；browser 下有 caches/snapshot。其布局符合包含用户文件和运行时配置的卷，但不据此判定具体软件框架或厂商。wallpaper.dat 等可能是用户设置资源，不能当成出厂 UI 素材。

未读取配置、媒体、短信或缓存文件正文，未导出联系人/短信。directory-layout.json 中浏览器快照的用户内容路径已脱敏。用户读写限制虽已撤销，本次仍只保留对设备结构必要的元数据。

尝试以普通用户权限读取 /dev/rdisk4 首块，返回 Permission denied；没有执行 sudo、没有生成 usb-volume-sector0.bin。BPB/魔数尚未独立验证，FAT12 结论来自 diskutil。

05:23 前已核对 USB/容量/介质名，再执行 diskutil unmountDisk /dev/disk4；返回成功，为拔线/不装电池的启动模式重试做准备。磁盘号可能变化，后续操作必须重新核对。

## 下一步修正

开源作者在 UMS9117 讨论中指出，应检查不装电池时的 USB 枚举；装电池可能令芯片提前上电。另一个 Nokia 110 4G TA-1395 用户在移除电池后得到 1782:4d00，并报告左软键有效。该型号不是 TA-1278，不能套用其固件大小、分区或加载器。TA-1278 的左软键依据仍是 FPDoom 型号测试表。

来源：https://github.com/ilyakurdyukov/spreadtrum_flash/issues/43#issuecomment-5249968147

本次修正前两次“装回电池再按左软键”的建议，下一次保留电池在机外、按左软键接 USB，仅观察枚举。
