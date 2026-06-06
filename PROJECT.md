# zed-config-sync

用于在多台机器之间同步 [Zed 编辑器](https://zed.dev) 配置的命令行工具。配置文件通过 GitHub Gist 存储，支持推送、拉取、查看历史版本、恢复指定版本。

---

## 目录

- [功能概览](#功能概览)
- [安装与构建](#安装与构建)
- [快速开始](#快速开始)
- [命令参考](#命令参考)
- [存储格式](#存储格式)
- [项目架构](#项目架构)
- [模块说明](#模块说明)
- [依赖说明](#依赖说明)

---

## 功能概览

| 功能 | 说明 |
|------|------|
| 推送配置 | 将本地 Zed 配置上传到 GitHub Gist，自动保留最近 5 个版本 |
| 拉取配置 | 从 Gist 下载最新版本并覆盖本地配置 |
| 历史记录 | 查看所有历史配置版本及上传机器名 |
| 版本恢复 | 将指定历史版本恢复到本地 |
| Token 管理 | 配置和验证 GitHub Personal Access Token |
| Gist 管理 | 配置和验证目标 Gist ID |

---

## 安装与构建

**前置要求：** Rust 工具链（edition 2024）

```bash
git clone <repo>
cd zed-config-sync
cargo build --release
```

构建产物位于 `target/release/zed-config`（Windows 下为 `zed-config.exe`）。

将可执行文件添加到 `PATH` 后即可全局使用。

---

## 快速开始

```bash
# 1. 设置 GitHub Token（需要 gist 读写权限）
zed-config set token ghp_xxxxxxxxxxxx

# 2. 设置目标 Gist ID
zed-config set gist <your-gist-id>

# 3. 推送本地配置到云端
zed-config push

# 4. 在另一台机器上拉取配置
zed-config pull
```

---

## 命令参考

### `zed-config token`

输出当前保存的 GitHub Token。若未设置则输出 `(not set)`。

```bash
zed-config token
```

---

### `zed-config gist`

输出当前保存的 Gist ID。若未设置则输出 `(not set)`。

```bash
zed-config gist
```

---

### `zed-config set token <TOKEN>`

设置 GitHub Personal Access Token。设置前会调用 GitHub API 验证 Token 有效性，验证失败则不保存。

```bash
zed-config set token ghp_xxxxxxxxxxxx
```

**Token 需要的权限：** `gist`（读写）

**输出示例：**
```
Validating token... valid.
Token saved.
```

---

### `zed-config set gist <GIST_ID>`

设置目标 Gist ID。执行前会依次验证：
1. Token 已配置
2. Token 有效
3. Gist ID 存在且可访问

三项验证均通过后才保存。

```bash
zed-config set gist abc123def456
```

---

### `zed-config push`

将本地 Zed 配置推送到云端。执行流程：

1. 拉取云端最新配置快照（如有），解压到临时目录
2. 将本地 `AppData\Roaming\Zed` 中的所有文件覆盖合并到临时目录（本地优先）
3. 将临时目录打包为 zip 文件
4. 若云端已有 5 个配置，先删除最旧的（同步更新 `history.json`）
5. 将新 zip 上传到 Gist，同步更新 `history.json`
6. 清理本地临时文件

```bash
zed-config push
```

**输出示例：**
```
Downloading latest config: cfg_2025-06-06_14-30
Packaging as cfg_2025-06-06_15-00...
Uploading cfg_2025-06-06_15-00...
Done! Config pushed successfully.
```

---

### `zed-config pull`

从云端拉取最新配置覆盖到本地。执行流程：

1. 获取云端配置列表，找到最新的快照
2. 下载并解压到临时目录
3. 将临时目录中的所有文件覆盖写入本地 `AppData\Roaming\Zed`（仅覆盖/新增，不删除本地多余文件）
4. 清理本地临时文件

```bash
zed-config pull
```

**注意：** 若云端无配置则提示退出；若本地 Zed 目录不存在则报错退出。

---

### `zed-config history`

列出 Gist 中所有历史配置记录，按日期从旧到新排序。

```bash
zed-config history
```

**输出示例：**
```
2025-06-01_09:00    DESKTOP-A1B2C3
2025-06-03_14:30    LAPTOP-XYZ
2025-06-06_15:00    DESKTOP-A1B2C3
```

每行格式：`YYYY-MM-DD_HH:mm    机器名`

---

### `zed-config restore <CONFIG_ID>`

将指定历史版本恢复到本地，执行流程与 `pull` 相同，区别是下载的是指定版本而非最新版本。

`CONFIG_ID` 从 `history` 命令输出中复制日期部分，格式为 `YYYY-MM-DD_HH:mm`。

```bash
zed-config restore 2025-06-03_14:30
```

若指定版本不存在于 Gist，则报错退出。

---

### `zed-config version`

输出当前工具版本号。

```bash
zed-config version
# 输出: 0.1.0
```

---

## 存储格式

### 配置文件目录

工具的配置文件存储在**可执行文件同级目录**下的 `zed-config-sync/` 目录中（首次 `set` 时自动创建）：

```
<exe所在目录>/
  zed-config-sync/
    config.json        ← Token 和 Gist ID
```

`config.json` 格式：

```json
{
  "github_token": "ghp_xxxxxxxxxxxx",
  "gist_id": "abc123def456"
}
```

### Gist 中的文件结构

```
<your-gist>/
  cfg_2025-06-01_09-00    ← 配置快照（Base64 编码的 zip）
  cfg_2025-06-03_14-30
  cfg_2025-06-06_15-00
  history.json            ← 历史记录索引
```

#### 配置快照文件

- **命名规则：** `cfg_YYYY-MM-DD_HH-MM`（无扩展名）
- **内容：** Zed 配置目录下所有文件打包成 zip 后经 Base64 编码的文本
- **最大数量：** 5 个，超出时自动删除最旧的
- **字符集：** 文件名中使用 `-` 代替 `:`，避免跨平台文件名问题

#### history.json

记录每个配置快照的上传信息，格式为键值对：

```json
{
  "cfg_2025-06-01_09-00": "DESKTOP-A1B2C3",
  "cfg_2025-06-03_14-30": "LAPTOP-XYZ",
  "cfg_2025-06-06_15-00": "DESKTOP-A1B2C3"
}
```

- **键：** 配置快照的完整文件名（含 `cfg_` 前缀）
- **值：** 上传时的机器名（优先读取 `COMPUTERNAME` 环境变量，其次执行 `hostname` 命令，均失败则为空字符串）
- **存储格式：** 明文 JSON，非 Base64 编码

### 本地临时目录

`push`/`pull`/`restore` 执行期间会在 `zed-config-sync/` 下创建临时目录，操作完成后自动清理：

```
zed-config-sync/
  zip/        ← 下载的 zip 文件（解码后）
  temp/       ← 解压/合并后的配置文件
```

---

## 项目架构

```
src/
  main.rs       ← CLI 定义（clap）及命令分发
  config.rs     ← 本地配置读写
  gist.rs       ← GitHub Gist API 封装及验证函数
  util.rs       ← 通用工具：Workspace、文件操作、zip 处理
  push.rs       ← push 命令实现
  pull.rs       ← pull 命令实现
  restore.rs    ← restore 命令实现
  history.rs    ← history 命令实现
```

### 模块依赖关系

```
main.rs
  ├── config.rs
  ├── gist.rs ──→ config.rs
  ├── util.rs ──→ config.rs, gist.rs
  ├── push.rs ──→ gist.rs, util.rs
  ├── pull.rs ──→ gist.rs, util.rs
  ├── restore.rs ──→ gist.rs, util.rs
  └── history.rs ──→ gist.rs
```

---

## 模块说明

### `config.rs` — 本地配置

管理存储在 `zed-config-sync/config.json` 中的工具配置。

| 方法 | 说明 |
|------|------|
| `Config::dir()` | 返回配置目录路径（可执行文件同级的 `zed-config-sync/`） |
| `Config::load()` | 读取配置文件，不存在或解析失败返回 `None` |
| `Config::load_or_default()` | 读取配置，不存在则返回空配置 |
| `Config::save()` | 写入配置文件，目录不存在时自动创建 |

---

### `gist.rs` — GitHub Gist API

封装对 GitHub Gist REST API 的所有操作。

**`GistClient` 方法：**

| 方法 | 说明 |
|------|------|
| `list_cfg_files()` | 列出 Gist 中所有 `cfg_` 前缀的文件，按名称升序排列 |
| `download_file(name)` | 下载指定文件，Base64 解码后返回原始字节 |
| `get_history()` | 获取 `history.json` 内容，文件不存在返回空 map |
| `patch_files(ops)` | 单次 PATCH 请求同时操作多个文件（`None` 表示删除） |

**独立函数：**

| 函数 | 说明 |
|------|------|
| `client_from_config()` | 从本地配置加载并构建 `GistClient`，Token 或 Gist ID 未设置时返回错误 |
| `validate_token(token)` | 调用 `GET /user` 验证 Token 有效性 |
| `validate_gist(token, gist_id)` | 调用 `GET /gists/{id}` 验证 Gist 可访问性 |

**二进制文件的处理：** Gist 文件内容只支持文本，zip 文件在上传前 Base64 编码，下载后 Base64 解码，还原为原始字节。下载时通过 `raw_url` 字段拉取完整内容，避免 Gist API 对大文件的截断问题。

---

### `util.rs` — 通用工具

**`Workspace` 结构体：** 管理 `push`/`pull`/`restore` 操作期间的临时工作目录。

| 方法 | 说明 |
|------|------|
| `Workspace::prepare()` | 清理并重建 `zip/` 和 `temp/` 临时目录 |
| `ws.cleanup()` | 删除临时目录 |
| `ws.download_and_extract(client, name)` | 下载指定文件到 `zip/`，解压到 `temp/` |

**工具函数：**

| 函数 | 说明 |
|------|------|
| `zed_dir()` | 返回本地 Zed 配置目录路径（`AppData\Roaming\Zed`） |
| `machine_name()` | 获取机器名，依次尝试 `COMPUTERNAME` 环境变量 → `hostname` 命令 → 空字符串 |
| `copy_dir_all(src, dst)` | 递归复制目录，同名文件覆盖 |
| `create_zip(src_dir, zip_path)` | 递归打包目录为 zip（使用 Deflate 压缩） |
| `extract_zip(zip_path, dst_dir)` | 解压 zip 文件，自动创建父目录 |
| `cleanup_dir(path)` | 删除目录（存在才删），错误时附带路径信息 |

---

### `push.rs` — 推送命令

执行步骤：

```
1. client_from_config()           → 验证配置
2. Workspace::prepare()           → 初始化临时目录
3. list_cfg_files()               → 获取云端文件列表
4. download_and_extract(latest)   → 下载最新快照到 temp/（如有）
5. copy_dir_all(zed_dir → temp/)  → 本地配置覆盖合并到 temp/
6. create_zip(temp/ → zip/*.zip)  → 打包
7. get_history()                  → 获取 history.json
8. 若数量 ≥ 5：patch_files({ oldest: null, history.json: updated })
9. patch_files({ new_cfg: b64, history.json: updated })
10. ws.cleanup()                  → 清理临时目录
```

步骤 8（删除旧版本）和步骤 9（上传新版本）各自通过单次 PATCH 请求同时更新 `history.json`，确保配置快照与历史记录的一致性。

---

### `pull.rs` — 拉取命令

```
1. client_from_config()
2. Workspace::prepare()
3. list_cfg_files() → 取最新文件名，无则提示退出
4. download_and_extract(latest)
5. 检查 zed_dir() 存在，否则清理后报错退出
6. copy_dir_all(temp/ → zed_dir)  → 仅覆盖/新增，不删除本地多余文件
7. ws.cleanup()
```

---

### `restore.rs` — 恢复命令

```
1. 解析参数：YYYY-MM-DD_HH:mm → cfg_YYYY-MM-DD_HH-MM
2. client_from_config()
3. Workspace::prepare()
4. list_cfg_files() → 确认目标文件存在，否则清理后报错退出
5. download_and_extract(target)
6. 检查 zed_dir() 存在，否则清理后报错退出
7. copy_dir_all(temp/ → zed_dir)
8. ws.cleanup()
```

---

### `history.rs` — 历史命令

```
1. client_from_config()
2. get_history() → 获取 history.json
3. 过滤并解析所有 cfg_YYYY-MM-DD_HH-MM 格式的键（格式不符则跳过）
4. 按日期升序排列
5. 格式化输出：YYYY-MM-DD_HH:mm    机器名
```

---

## 依赖说明

| 依赖 | 版本 | 用途 |
|------|------|------|
| `clap` | 4 | CLI 参数解析（derive 宏） |
| `serde` + `serde_json` | 1 | JSON 序列化/反序列化 |
| `ureq` | 2 | HTTP 客户端（同步，调用 GitHub API） |
| `anyhow` | 1 | 错误处理与传播 |
| `zip` | 2 | zip 压缩与解压 |
| `chrono` | 0.4 | 日期时间解析与格式化 |
| `base64` | 0.22 | zip 二进制内容的 Base64 编解码 |
| `dirs` | 5 | 跨平台获取用户数据目录 |
