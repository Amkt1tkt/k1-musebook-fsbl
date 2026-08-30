# K1 MUSE Book FSBL

用 Rust 为 SpacemiT K1 MUSE Book 实现的二级引导程序（SPL / FSBL），以及配套的 USB 烧录工具链。

```
BROM → k1-musebook-fsbl → SBI → Kernel
```

---

- [启动链](#启动链)
- [快速开始](#快速开始)
- [命令参考](#命令参考)
- [项目结构](#项目结构)
- [地址布局](#地址布局)
- [布局调整](#布局调整)
- [镜像格式](#镜像格式)
- [参考](#参考)

---

## 启动链

```
BROM（片内 ROM，不可改）
  │ 读 NOR @ 0x00000 的 80 字节 bootinfo，取出 spl0_offset / spl1_offset / spl_size_limit
  │ 从 NOR @ 0x20000 读 FSBL，校验 AIHD 头与 RSA-2048 签名
  │ 拷入 SRAM 0xC0800000，跳转到 0xC0801000
  ▼
k1-musebook-spl（运行在 SRAM，约 50 KiB）
  │ hart 0：
  │   1. 立栈、清 BSS
  │   2. UART 日志、M-mode trap handler、Generic Counter
  │   3. I2C8 → SPM8821 把 VDD_CORE 抬到 1.05 V
  │   4. PLL3 → 双 cluster 升频到 1600 MHz
  │   5. LPDDR4X 两遍初始化 + 1200 / 1600 / 2400 MT 逐级训练 + 图案自检
  │   6. 使能 D/I-cache、BPU、预取、L2 snoop
  │   7. PCIe Port C 拉成 RC（PUPHY → Gen2 LTSSM L0 → iATU → NVMe BAR）
  │   8. 解析 GPT，按名字把 sbi / kernel / dtb / initramfs 读进 DDR
  │   9. 把两个 cluster 都并入 CCI 一致性域，唤醒 hart 1-7
  │ hart 1-7：自旋等 hart 0 完成，然后各自开 cache 与性能特性
  │
  │ 全部 hart 跳转到 SBI：a0 = hartid, a1 = DTB 地址, a2 = &fw_dynamic_info
  ▼
SBI（DDR 0x0008_0000）
  ▼
Kernel（DDR 0x0020_0000，S 模式）
```


---

## 快速开始

### 环境依赖

只需[安装 Rust 工具链](https://rust-lang.org/tools/install/)。

Linux 下访问 USB 需要 udev 规则或 `sudo`（设备 VID:PID 为 `361c:1001`）。

### 构建

```sh
cargo xtask build
```

产物写在 `images/`：

| 文件 | 说明 |
| --- | --- |
| `images/k1-musebook-spl-fsbl.bin` | 签名后的 SPL，烧到 NOR `0x20000` |
| `images/k1-musebook-flash-server-fsbl.bin` | 签名后的板端烧录服务，由 BROM fastboot 临时下载执行 |
| `images/bootinfo.bin` | 80 字节 bootinfo，烧到 NOR `0x0` |

### 烧录

1. 用 USB 连接主机与 MUSE Book 左侧的 OTG 接口。
2. 使用取卡针顶住 MUSE Book 右侧的下载模式孔不放，同时按开机键。
3. 此时 MUSE Book 应该会进入 BROM fastboot 模式（USB 出现 `361c:1001`）。
4. 主机按需执行烧录命令：

```sh
# NOR @ 0x0：烧录 bootinfo
cargo xtask bootinfo flash

# NOR @ 0x20000：烧录 SPL
cargo xtask flash nor flash

# SSD：按 spl/src/layout.rs 建立 GPT 分区表
cargo xtask flash gpt init

# SSD：烧入各阶段镜像
cargo xtask flash gpt flash \
    sbi=./sbi.bin \
    kernel=./kernel.bin \
    dtb=./dtb.bin \
    initramfs=./initramfs.cpio.gz \
    rootfs=./rootfs.ext2

# 确认 GPT 分区表
cargo xtask flash gpt list
```

---

## 命令参考

### `cargo xtask`

| 命令 | 作用 |
| --- | --- |
| `cargo xtask build` | 编译 SPL 与 flash-server → 提取 `PT_LOAD` → RSA 签名 → 写 `images/*-fsbl.bin`；同时打包 `images/bootinfo.bin` |
| `cargo xtask bootinfo flash [CONFIG]` | 按 `CONFIG`（默认 `bootinfo.toml`）打包 bootinfo 并写入 NOR `0x0` |
| `cargo xtask bootinfo read [OUT]` | 从 NOR `0x0` 读回 80 字节，反解析成 TOML 写到 `OUT`（默认 `./bootinfo-out.toml`，同时留下 `./bootinfo-out.bin`） |
| `cargo xtask flash <ARGS…>` | 编译 flash-server，然后把 `ARGS` 原样转交给主机端 CLI |

### flash CLI

以下子命令写在 `cargo xtask flash` 后面。

全局参数 `--server-image <PATH>` 指定上传并与主机通信的板端镜像，默认 `./images/k1-musebook-flash-server-fsbl.bin`。

| 命令 | 说明 |
| --- | --- |
| `ping` | 握手，打印 flash-server 的 ICD 版本（`0x00010000`） |
| `nor flash [OFFSET] [FILE]` | 先擦除 4 KiB 对齐的覆盖窗口再写入。默认 `OFFSET=0x20000`、`FILE=./images/k1-musebook-spl-fsbl.bin` |
| `nor read <OFFSET> <LEN> [OUT]` | 读 NOR，默认写到 `./nor-read-out.bin` |
| `nvme flash <LBA> <FILE>` | 从指定 LBA 起写入文件（末尾按 512 字节用 `0xFF` 补齐） |
| `nvme read <LBA> <LEN> <OUT>` | 从指定 LBA 读 `LEN` 字节 |
| `gpt list` | 解析主 GPT，列出所有分区的名字 / 起止 LBA / 大小 |
| `gpt init [--disk-lba-count N]` | 按 `spl/src/layout.rs` 写入保护 MBR + 主 GPT + 备份 GPT。磁盘容量优先从已有备份头推断，推断不出时用 `--disk-lba-count` 指定 |
| `gpt flash NAME=FILE …` | 按分区名写入文件 |

---

## 项目结构

| Crate | 目标 | 职责 |
| --- | --- | --- |
| `spl` (`k1-musebook-spl`) | RISC-V 固件 + 库 | SPL 本体，同时把全部硬件驱动、`layout.rs`、`gpt.rs` 作为库导出 |
| `flash/server` (`k1-musebook-flash-server`) | RISC-V 固件 | 板端烧录服务，复用 `spl` 的 DDR / PCIe / NVMe 驱动，自己加 QSPI NOR 与 USB |
| `flash/client` (`k1-musebook-flash-client`) | 主机 | 烧录 CLI，依赖 `spl`（共享分区布局）与 `flash/server`（共享 RPC ICD） |
| `xtask` | 主机 | 构建、ELF 转裸二进制、RSA 签名、bootinfo 打包、烧录流程编排 |


### SPL 模块（`spl/src/`）

| 文件 | 职责 |
| --- | --- |
| `main.rs` | 复位入口、hart 分流、清 BSS、`boot()` 主流程、跳转 SBI、panic handler |
| `lib.rs` | 模块清单与跨模块 `use` 的汇聚点 |
| `layout.rs` | GPT 分区表、DDR 加载地址、NVMe DMA 窗口 |
| `mmio.rs` | `MMIO<T>`（把基地址视作类型化寄存器块）与 `Raw`（按字节偏移读写 `u32`） |
| `log.rs` / `uart.rs` | `log` crate 的 UART 后端；16550 轮询发送，换行自动补 CR |
| `trap.rs` | 屏蔽全部中断，装 Direct 模式 `mtvec`，任何 trap 打印 `mcause`/`mepc`/`mtval` 后 panic |
| `time.rs` | Generic Counter（`0xD5001000`）与忙等 `sleep` |
| `pinmux.rs` | MFPR（`0xD401E000`）：本固件用到的 QSPI 与 I2C 引脚 |
| `pcr.rs` | 电源 / 时钟 / 复位寄存器块：`APMU` / `APBC` / `APBS` / `MPMU` |
| `i2c.rs` | TWSI8（`0xD401D800`）7 位地址主机写 |
| `cci.rs` | CCI snoop / DVM，跨 cluster 缓存一致性 |
| `cpu/` | 调压（`voltage`）、升频（`freq`）、cache（`cache`）、BPU/预取/snoop（`perf`）、副核唤醒（`multicore`）、X60 自定义 CSR（`csr`） |
| `ddr/` | LPDDR4X 拉起：`clock` / `ctrl` / `phy` / `dfi` / `dram` / `byte` / `freq` / `train` / `image`（训练固件 blob）/ `verify` |
| `pcie/` | Port C RC：`clock` / `phy` / `link`（Gen2 LTSSM）/ `atu`（iATU 窗口）/ `bar` |
| `nvme/` | admin + I/O 队列建立、4 KiB 分块读写、DMA cache 维护 |
| `gpt.rs` | 从 LBA 1 解析 GPT，按 UTF-16 名字索引分区，加载到 DDR 后 `cbo.clean` |
| `handoff.rs` | OpenSBI `fw_dynamic_info` v2 结构 |

---

## 地址布局

### SSD GPT 分区与 DDR 加载地址

全部定义在 [`spl/src/layout.rs`](spl/src/layout.rs)。

| 分区名 | 起始 LBA | 分区大小 | DDR 加载地址 | DDR 窗口上限 |
| --- | --- | --- | --- | --- |
| `sbi` | 2048 | 512 KiB | `0x0008_0000` | 1 MiB |
| `kernel` | 4096 | 12 MiB | `0x0020_0000` | 14 MiB |
| `dtb` | 28672 | 256 KiB | `0x0100_0000` | 1 MiB |
| `initramfs` | 32768 | 64 MiB | `0x0800_0000` | 64 MiB |
| `rootfs` | 163840 | 磁盘剩余全部 | 不加载 | — |


DDR 上还有两块非分区用途的区域：

| 地址 | 用途 |
| --- | --- |
| `0x0001_0000` | DDR 训练结束后的图案自检缓冲（512 字节） |
| `0x0400_0000` | NVMe DMA 窗口（28 KiB）：admin SQ/CQ、I/O SQ/CQ、读写 PRP1/PRP2 |

flash-server 运行时还会额外用到 DDR：

| 地址 | 用途 |
| --- | --- |
| `0x0500_0000` | USB RX 缓冲（1 MiB + 4 KiB） |
| `0x0510_1000` | USB TX 缓冲（1 MiB + 4 KiB） |
| `0x1000_0000` | 进入 RPC 监听前切换到的 DDR 栈顶 |

### PCIe 地址窗口

| 窗口 | CPU 地址 | 大小 | 说明 |
| --- | --- | --- | --- |
| CFG | `0xA000_0000` | 1 MiB | outbound iATU region 0，TLP 类型 CFG0，target bus 1 dev 0 fn 0 |
| MEM | `0xA200_0000` | 352 MiB | outbound iATU region 1，1:1 直通；NVMe BAR0 就指到这里 |

NVMe 控制器寄存器的 MMIO 基址因此是 `0xA200_0000`（`NVME_CTRL_BASE`），doorbell 在 `+0x1000`。

### SRAM

K1 的 SRAM 只有 256 KiB（`0xC0800000`–`0xC0840000`），两个固件各有一份链接脚本。

**SPL**（[`spl/linker-script.spl.ld`](spl/linker-script.spl.ld)）：

| 范围 | 用途 |
| --- | --- |
| `0xC080_0000`–`0xC080_1000` | FSBL 头（4 KiB，由 `xtask` 生成，不属于链接脚本） |
| `0xC080_1000`–`0xC083_4000` | `.text` / `.rodata` / `.data` |
| `0xC083_7000`–`0xC083_9000` | `.bss`（8 KiB） |
| `0xC083_9000`–`0xC084_0000` | 栈（28 KiB，`STACK_TOP = 0xC0840000`，向下生长） |

**flash-server**（[`flash/server/linker-script.flash.ld`](flash/server/linker-script.flash.ld)）：

因为要复用 BROM 的 USB ROM 函数，必须同时避开训练固件区和 BROM 自己的全局变量 / 栈
（`0xC083_8000`–`0xC084_0000`），所以整体往下压：

| 范围 | 用途 |
| --- | --- |
| `0xC080_0000`–`0xC080_1000` | FSBL 头 |
| `0xC080_1000`–`0xC082_1000` | `.text` / `.rodata` / `.data`（128 KiB） |
| `0xC082_1000`–`0xC082_3000` | `.bss`（8 KiB） |
| `0xC082_3000`–`0xC083_2000` | 栈（60 KiB，`STACK_TOP = 0xC0832000`） |

USB 的 1 MiB 收发缓冲放不进 SRAM，所以 flash-server 在进入 RPC 循环前会把 `sp` 挪到 DDR。

### QSPI NOR

由 [`bootinfo.toml`](bootinfo.toml) 描述，容量 1 MiB：

| 偏移 | 内容 |
| --- | --- |
| `0x00000` | bootinfo（80 字节，BROM 直接解析） |
| `0x20000` | 主 FSBL（`spl0_offset`） |
| `0x70000` | 备份 FSBL（`spl1_offset`，主槽校验失败时 BROM 回退到这里） |

SPL 跑起来之后 NOR 就不再参与任何数据加载。

---

## 布局调整

### 改分区布局或加载地址

编辑 [`spl/src/layout.rs`](spl/src/layout.rs)。

```rust
pub const KERNEL: GptPart = GptPart {
    name: "kernel",        // GPT 分区名（UTF-16 匹配）
    lba_start: 4096,       // 磁盘起始 LBA
    lba_max: 24576,        // 分区占用的 LBA 数
    load_base: 0x0020_0000, // 拷进 DDR 的地址
    load_max: 0x00E0_0000,  // 这个 DDR 窗口的上限
};
```

### 改 NOR 布局

编辑 [`bootinfo.toml`](bootinfo.toml)，然后 `cargo xtask bootinfo flash`。
`spl0_offset` / `spl1_offset` 会被校验不与 bootinfo 自身所在的首扇区重叠、两个槽位不互相重叠。

### 改 SRAM 布局

编辑对应的链接脚本。注意两个硬约束：训练固件区、以及 flash-server 必须避开的 BROM 区。

---

## 镜像格式

### FSBL

由 [`xtask/src/fsbl.rs`](xtask/src/fsbl.rs) 生成：4 KiB 头 + 32 字节对齐的裸镜像 + 256 字节签名。

| 偏移 | 长度 | 内容 |
| --- | --- | --- |
| `0x000` | 256 | ROTPK（RSA-2048 公钥模数，大端） |
| `0x100` | 32 | header0（`AIHD` 魔数 + version=1 + 证书区长度 `0x1000`） |
| `0x120` | 480 | keydata（4 张 key table：`spl`/`uboot`/`kernel`/`rootfs` + ROTPK 的 SHA-256） |
| `0x300` | 2048 | oem_key（slot 0 放签名公钥模数，其余为零） |
| `0xB00` | 256 | signature0 = RSA-PKCS#1v1.5-SHA256(header0 ‖ keydata ‖ oem_key) |
| `0xC00` | 992 | 填充 |
| `0xFE0` | 32 | header1（`AIHD` + 裸镜像长度） |
| `0x1000` | — | 裸镜像（ELF 的 `PT_LOAD` 段按物理地址拼接，空洞补零） |
| 末尾 | 256 | signature1 = RSA-PKCS#1v1.5-SHA256(header1 ‖ 裸镜像) |

### bootinfo

由 [`xtask/src/bootinfo.rs`](xtask/src/bootinfo.rs) 生成，共 `0x50` 字节：`0x40` 字节头 + CRC32 + 12 字节填充。

| 偏移 | 字段 | 说明 |
| --- | --- | --- |
| `0x00` | magic | `0xB00714F0` |
| `0x04` | version | `0x00010001` |
| `0x08` | flash_type | `NORF` |
| `0x10` | `page_size` | NOR 页大小（256） |
| `0x14` | `block_size` | NOR 擦除块大小（`0x10000`） |
| `0x18` | `total_size` | NOR 容量（`0x100000`） |
| `0x20` | `spl0_offset` | 主 FSBL 偏移（`0x20000`） |
| `0x24` | `spl1_offset` | 备份 FSBL 偏移（`0x70000`） |
| `0x28` | `spl_size_limit` | BROM 允许的最大 FSBL 大小（`0x36000`） |
| `0x2C` | `partitiontable0_offset` | 未使用，保留 BROM 布局 |
| `0x30` | `partitiontable1_offset` | 未使用，保留 BROM 布局 |
| `0x40` | crc32 | 前 `0x40` 字节的 IEEE CRC32 |

### 烧录协议

主机与板端共用 [`flash/server/src/protocol.rs`](flash/server/src/protocol.rs) 里的 `postcard-rpc` ICD：

| Endpoint | Path | 请求 / 响应 |
| --- | --- | --- |
| `PingEndpoint` | `ping` | `()` → `u32`（版本 `0x00010000`） |
| `NorEraseEndpoint` | `nor/erase` | `NorRange` → `Result<(), FlashServerError>` |
| `NorWriteEndpoint` | `nor/write` | `NorChunk` → `Result<(), _>` |
| `NorReadEndpoint` | `nor/read` | `NorRange` → `Result<ByteBuf, _>` |
| `NvmeWriteEndpoint` | `nvme/write` | `NvmeChunk` → `Result<(), _>` |
| `NvmeReadEndpoint` | `nvme/read` | `NvmeRange` → `Result<ByteBuf, _>` |

单次载荷上限 1 MiB。底层走 BROM 已经枚举好的 USB bulk 端点（OUT `0x02` / IN `0x81`），
主机在每个 RPC 帧前先发一个 512 字节的包，头 4 字节是小端帧长。

板端并没有自己实现 USB 驱动，而是直接调用 BROM ROM 里的收发函数（`0xFFE037B6` / `0xFFE038D0`）。
代价是必须保住 BROM 的 `gp` 和全局区——这就是 flash-server 入口第一件事是恢复
`gp = 0xC0838C10`、链接脚本又要避开 `0xC0838000` 以上区域的原因。
另外 DDR 初始化会打断 USB 控制器，所以 `usb::init()` 会清掉 BROM 的 `g_usb_ready` 并重跑
`controller_run` 触发重新枚举，主机端的 `wait_usb_reenumerate` 与之配合。

---

## 参考

- [SpacemiT K1 官方 BSP uboot-2022.10](https://github.com/spacemit-com/uboot-2022.10)
- [SpacemiT K1 芯片文档](https://github.com/spacemit-com/docs-chip)
