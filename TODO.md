# YT-DLP Desktop 开发进度 TodoList

> 最后更新: 2026-03-25 | 当前版本: v0.2.x

---

## 📊 总体进度

| 模块 | 完成度 | 状态 |
|------|--------|------|
| 基础架构 | 100% | ✅ 完成 |
| 下载核心 | 100% | ✅ 完成 |
| 播放列表 | 100% | ✅ 完成 |
| 下载队列 | 100% | ✅ 完成 |
| 用户界面 | 100% | ✅ 完成 |
| 设置系统 | 100% | ✅ 完成 |
| 通知系统 | 100% | ✅ 完成 |
| 系统托盘 | 100% | ✅ 完成 |
| 剪贴板监听 | 100% | ✅ 完成 |
| 更新系统 | 100% | ✅ 完成 |
| 数据导出/导入 | 100% | ✅ 完成 |
| 国际化 | 90% | 🔄 基本完成 |
| 主题系统 | 90% | 🔄 基本完成 |
| Cookie 诊断 | 100% | ✅ 完成 |
| 暂停/恢复 | 100% | ✅ 完成 |
| 批量下载 | 100% | ✅ 完成 |

---

## ✅ 已完成功能

### 1. 基础架构
- [x] Tauri 2 + Vue 3 + Rust 项目搭建
- [x] Vite 构建配置
- [x] TypeScript 类型定义
- [x] Pinia 状态管理集成 (stores/download.ts, stores/settings.ts)
- [x] Tauri 权限配置 (capabilities/default.json)
- [x] 自定义标题栏 (TitleBar.vue，无原生装饰)
- [x] 窗口控制（最小化/最大化/关闭）
- [x] CSS 变量主题系统 (main.css)
- [x] 暗色模式自动适配 (prefers-color-scheme)
- [x] 页面 Tab 切换动画 (fade transition)
- [x] 全局滚动条样式美化

### 2. 下载核心功能
- [x] URL 输入框 (UrlInput.vue)
  - [x] 粘贴/清除/Enter 快捷键
  - [x] 多行 URL 批量粘贴自动识别
  - [x] 从剪贴板粘贴按钮
  - [x] 粘贴后自动触发解析
  - [x] 前端 URL 预校验 (isLikelyUrl 判断)
- [x] yt-dlp 视频信息解析 (--dump-json --no-download)
- [x] 视频预览卡片 (VideoPreview.vue)
  - [x] 缩略图/标题/时长/上传者/日期
  - [x] HTTP→HTTPS 缩略图自动转换
  - [x] 缩略图加载失败占位符
- [x] 格式列表解析（按分辨率去重，高→低排序）
- [x] 画质选择下拉框 (DownloadOptions.vue)
- [x] 仅音频模式 (MP3 提取，-x --audio-format mp3)
- [x] 音频质量选择（最佳/高/中）
- [x] 字幕下载选项（中文/英文/日文）
- [x] 字幕嵌入 (--embed-subs)
- [x] 下载目录选择器（Tauri dialog）
- [x] 下载任务启动
- [x] 实时进度解析（百分比/速度/ETA/文件大小）
- [x] 进度条组件 (ProgressBar.vue)
- [x] 合并状态检测 ([Merger]/[ffmpeg]/[ExtractAudio])
- [x] 下载完成状态处理
- [x] 下载失败错误捕获
- [x] 输出文件路径捕获 (--print after_move:filepath)
- [x] ffmpeg 目录传递 (--ffmpeg-location)
- [x] Windows 文件名安全处理 (--windows-filenames)
- [x] UTF-8 编码强制 (PYTHONIOENCODING=utf-8)
- [x] Windows 隐藏命令行窗口 (creation_flags(0x08000000))
- [x] 速度限制 (--limit-rate)
- [x] 下载缩略图 (--write-thumbnail)
- [x] 下载元数据 (--write-info-json)
- [x] 额外自定义参数传递
- [x] 合并输出格式强制 MP4 (--merge-output-format mp4)
- [x] 多流下载进度自动调整 (Destination 计数机制)

### 3. 暂停/恢复下载
- [x] 暂停下载 (pause_download 命令)
  - [x] 杀进程 + 保留临时文件
  - [x] 暂停任务移入暂停列表 (queue.rs::move_to_paused)
  - [x] 恢复标记 (is_resume=true)
- [x] 恢复下载 (resume_download 命令)
  - [x] 从暂停列表取出任务
  - [x] 重新入队等待下载
  - [x] 断点续传 (-c 参数)
- [x] 暂停状态 UI 显示
- [x] 暂停/恢复按钮 (DownloadItem.vue)

### 4. 任务取消
- [x] 单任务取消按钮
- [x] 进程 PID 记录 (ProcessManager)
- [x] Windows 进程杀死 (taskkill /PID /F /T)
- [x] Unix 进程杀死 (SIGTERM)
- [x] 队列中任务取消 (直接移出队列)
- [x] 数据库状态更新为 Cancelled
- [x] 取消任务事件过滤 (cancelledIds 忽略列表)

### 5. 下载队列管理
- [x] 队列数据结构 (VecDeque<QueuedTask>) (queue.rs)
- [x] 并发数控制 (max_concurrent 配置)
- [x] 队列消费循环 (start_queue_worker)
- [x] 槽位等待机制 (Notify + wait_for_slot)
- [x] 任务完成自动释放槽位 (task_finished)
- [x] 动态调整并发数 (set_max_concurrent)
- [x] 队列状态查询 API (get_queue_status)
- [x] 前端队列状态展示
- [x] 排队中任务显示"排队等待中..."状态
- [x] 全部取消排队任务
- [x] tauri::async_runtime::spawn 使用

### 6. 播放列表支持
- [x] 播放列表检测 (--flat-playlist 多行 JSON 解析)
- [x] 播放列表条目解析 (parse_playlist_output)
- [x] 播放列表 JSON 格式解析 (parse_playlist_json)
- [x] 播放列表信息展示 (VideoPreview.vue)
- [x] 播放列表选择组件 (PlaylistSelector.vue)
  - [x] 全选/取消全选/单选
  - [x] 条目缩略图展示
  - [x] 条目时长显示
- [x] 批量下载 API (start_batch_download)
- [x] 批量任务创建（每个视频独立任务 ID）
- [x] 播放列表标记 (playlist_title/playlist_index/playlist_total)
- [x] 下载项显示播放列表序号标签
- [x] 数据库存储播放列表字段

### 7. 批量 URL 下载
- [x] 多行 URL 输入支持 (UrlInput.vue)
- [x] URL 预览组件 (BatchPreview.vue)
- [x] 批量下载模式切换 (单个/批量)
- [x] 批量 URL 入队 API (start_batch_urls)
- [x] URL 有效性检测
- [x] 批量任务创建（无预解析）
- [x] URL 缩短显示

### 8. 历史与队列管理
- [x] SQLite 数据库初始化 (database/mod.rs)
- [x] 数据库迁移（自动添加新列）
- [x] 下载任务持久化 (insert_task)
- [x] 任务状态更新 (update_task_status)
- [x] 历史记录列表（按创建时间倒序）
- [x] 活跃任务列表（分为"下载中"和"排队中"）
- [x] 队列状态面板（数字统计卡片）
- [x] 任务状态徽章（✅已完成/❌失败/⏹已取消）
- [x] 单条历史删除
- [x] 清空全部历史（含确认弹窗）
- [x] 打开文件所在文件夹 (跨平台 open_file_location)
- [x] 打开下载目录
- [x] 失败任务重试 (retry_download 命令)
- [x] 文件路径简短显示 (shortenPath)
- [x] 今日显示时间，非今日显示日期
- [x] 暂停任务在队列中显示

### 9. 错误处理系统
- [x] 友好错误面板 UI (UrlInput.vue)
- [x] 错误类型自动分类
  - [x] 无效 URL
  - [x] 不支持的网站
  - [x] Cookie/登录问题
  - [x] 网络错误
  - [x] 视频不可用
  - [x] 通用错误
  - [x] SSL 错误
  - [x] HTTP 状态码错误 (403/404/429)
  - [x] 磁盘空间不足
  - [x] 权限错误
- [x] 每种错误类型独立图标和标题
- [x] 针对性解决方案提示列表
- [x] 错误信息复制到剪贴板
- [x] 从错误面板跳转设置页 Cookie 区域
- [x] 重试按钮
- [x] Cookie 自动降级尝试 (edge→chrome→firefox→brave)
- [x] 抖音特殊处理 (Fresh cookies 重试策略)
- [x] 错误友好化显示 (errors.ts: friendlyDownloadError)

### 10. 设置系统
- [x] 设置页面完整 UI (SettingsView.vue)
- [x] 设置 JSON 持久化 (settings.json)
- [x] 自动保存（每次修改立即保存）
- [x] 手动保存按钮 + 时间戳反馈
- [x] 重置所有设置（含确认弹窗）
- [x] 通用设置
  - [x] 默认保存路径选择
  - [x] 同时下载数量 (1/2/3/5/8)
  - [x] 文件命名模板（3种预设）
  - [x] 语言切换（中文/English）
  - [x] 主题切换（亮色/暗色/跟随系统）
- [x] 下载设置
  - [x] 速度限制输入
  - [x] 音频质量选择
  - [x] 下载缩略图开关
  - [x] 下载元数据开关
- [x] 通知设置
  - [x] 下载完成通知开关
  - [x] 下载失败通知开关
  - [x] 最小化到托盘开关
  - [x] 剪贴板监听开关
- [x] 网络设置
  - [x] 代理模式（系统/自定义/无）
  - [x] 自定义代理地址输入（条件显示）
- [x] Cookie 设置
  - [x] Cookie 文件选择
  - [x] 浏览器 Cookie 读取 (Chrome/Edge/Firefox)
  - [x] 不使用 Cookie
  - [x] Cookie 配置状态指示
  - [x] 清除 Cookie 文件
  - [x] Cookie 获取教程（4 步教程）
  - [x] Cookie 诊断功能
  - [x] 滚动定位 + 高亮动画
- [x] 高级设置
  - [x] yt-dlp 版本检测
  - [x] yt-dlp 一键更新
  - [x] 更新结果反馈（成功/已是最新/失败）
  - [x] 额外参数输入
- [x] 数据管理
  - [x] 导出下载历史（JSON）
  - [x] 导出下载历史（CSV，含 UTF-8 BOM）
  - [x] 导入链接列表（txt 文件，支持注释行）
  - [x] 导出设置（JSON）
  - [x] 导入设置（JSON）
- [x] Toggle 开关组件样式

### 11. Cookie 诊断功能
- [x] Cookie 配置方式检测
- [x] 浏览器运行状态检测
- [x] Cookie 数据库路径检测
- [x] Cookie 文件存在性检测
- [x] Cookie 文件大小检测
- [x] yt-dlp 实际测试运行
- [x] 诊断结果展示面板
  - [x] 检查项列表（配置方式/浏览器状态/Cookie 数据库/Cookie 文件/实测结果）
  - [x] Cookie 数据库路径列表
  - [x] yt-dlp 测试输出
  - [x] 建议提示列表
- [x] App-Bound Encryption 检测与提示
- [x] Cookie 解密失败诊断

### 12. 桌面通知
- [x] 通知权限检查 (isPermissionGranted)
- [x] 通知权限请求 (requestPermission)
- [x] 下载完成通知
- [x] 下载失败通知
- [x] 通知开关控制
- [x] tauri-plugin-notification 集成

### 13. 系统托盘
- [x] 托盘图标创建 (TrayIconBuilder)
- [x] 托盘右键菜单（显示主窗口 / 退出）
- [x] 托盘左键单击显示窗口
- [x] 点击关闭按钮最小化到托盘 (on_window_event 拦截 CloseRequested)
- [x] 最小化到托盘可通过设置开关控制
- [x] 从托盘恢复窗口 (show + unminimize + set_focus)

### 14. 剪贴板监听
- [x] 跨平台剪贴板读取 (clipboard.rs)
  - [x] Windows: powershell Get-Clipboard
  - [x] macOS: pbpaste
  - [x] Linux: xclip / xsel
- [x] 定时轮询（1.5秒间隔）
- [x] 重复内容过滤（与上次内容比对）
- [x] 视频 URL 识别（20+ 主流视频网站域名匹配）
- [x] 检测到 URL 后 emit 事件到前端
- [x] 前端浮动提示组件 (ClipboardAlert.vue)
- [x] "解析"按钮 → 自动填入 URL 并开始解析
- [x] "关闭"按钮 → 忽略本次检测
- [x] 可通过设置开关启用/禁用
- [x] 启用/禁用状态实时同步 (set_clipboard_watch 命令)

### 15. yt-dlp 更新系统
- [x] 版本检测 (--version)
- [x] 一键更新 (yt-dlp -U)
- [x] 更新结果解析
- [x] 更新后自动刷新版本号
- [x] 更新中状态指示（按钮 disabled + 文案变更）
- [x] 更新结果 5 秒自动消失

### 16. 数据导出/导入
- [x] 导出历史为 JSON (serde_json::to_string_pretty)
- [x] 导出历史为 CSV (UTF-8 BOM + 中文表头)
- [x] CSV 字段转义（双引号处理）
- [x] 导入 URL 列表（txt 文件，支持 # 和 // 注释）
- [x] 导出设置为 JSON
- [x] 导入设置从 JSON（覆盖当前设置）
- [x] Tauri save dialog 选择导出路径
- [x] Tauri open dialog 选择导入文件
- [x] 操作结果提示（成功/失败）

### 17. 国际化 (i18n)
- [x] i18n 系统搭建 (i18n.ts，ref + computed 响应式)
- [x] 中文语言包（完整）
- [x] 英文语言包（完整）
- [x] t() 翻译函数
- [x] useI18n() composable
- [x] 语言切换（设置页下拉框）
- [x] 语言设置持久化
- [x] 加载设置时自动设置语言
- [x] SettingsView 全面使用 i18n key
- [x] QueueView 全面使用 i18n key
- [x] DownloadView 全面使用 i18n key
- [x] App.vue Tab 标签使用 i18n key
- [x] 批量下载组件使用 i18n key
- [ ] 部分组件 i18n 覆盖（UrlInput/DownloadOptions/VideoPreview/PlaylistSelector 等）

### 18. 主题系统
- [x] CSS 变量暗色/亮色自动切换 (prefers-color-scheme)
- [x] 设置中 theme 字段已存在
- [x] 手动切换亮色/暗色/跟随系统 UI (SettingsView.vue 主题选择器)
- [x] 主题切换后立即生效

### 19. 二进制管理
- [x] yt-dlp 路径多路径候选查找 (ytdlp/binary.rs)
  - [x] app_dir/bin/
  - [x] 可执行文件同目录
  - [x] binaries/ 目录
  - [x] CARGO_MANIFEST_DIR/binaries/
- [x] ffmpeg 路径查找（同上）
- [x] 系统 PATH 降级查找 (where / which)
- [x] 查找过程详细日志输出
- [x] 版本检测命令

### 20. 进度管理
- [x] ProcessManager 进程追踪
- [x] PID 记录与查找
- [x] 暂停任务集合 (paused_set)
- [x] 取消任务集合 (cancelled_set)
- [x] 取消事件忽略（5秒延迟清理）
- [x] 多流下载进度调整（Destination 计数机制）

---

## 🔄 部分完成 / 需要优化

### 1. 国际化全覆盖
- [x] 设置页完整 i18n
- [x] 队列页完整 i18n
- [x] 下载页完整 i18n
- [x] Cookie 教程完整 i18n
- [ ] 部分组件硬编码中文（UrlInput/DownloadOptions/VideoPreview/PlaylistSelector 等）

### 2. 进度解析优化
- [x] 百分比解析
- [x] 速度解析
- [x] ETA 解析
- [x] 文件大小解析
- [x] 多流下载进度调整（Destination 计数机制）
- [x] downloaded_size 字段存在（DisplayItem.vue 显示）

---

## 📋 待开发功能

### 1. 性能优化
- [ ] 虚拟列表（历史记录超过 100 条时使用虚拟滚动）
- [ ] 缩略图懒加载（Intersection Observer）
- [ ] 缩略图本地缓存（下载到 app_data 目录）
- [ ] 数据库查询分页优化（前端无限滚动加载）
- [ ] 日志缓冲大小限制（防止大文件下载时内存溢出）
- [ ] stderr 收集行数限制

### 2. 安全增强
- [ ] Cookie 文件路径加密存储
- [ ] 代理密码遮罩显示
- [ ] 下载目录写入权限预检测
- [ ] 输入内容 XSS 过滤

### 3. UI/UX 优化
- [ ] 移动端/小窗口适配（当前 minWidth: 640）
- [ ] 下载完成音效
- [ ] 拖拽文件到窗口触发下载
- [ ] 右键菜单（复制 URL / 复制标题 / 打开链接）
- [ ] 快捷键支持（Ctrl+V 自动解析 / Ctrl+D 开始下载）
- [ ] 下载速度图表（历史速度折线图）
- [ ] 磁盘空间检测提示

### 4. 应用自更新
- [ ] 检测应用新版本（GitHub Release API）
- [ ] 一键下载更新包
- [ ] 自动安装更新（Tauri updater plugin）

### 5. 高级播放列表
- [ ] 播放列表搜索过滤
- [ ] 按时长/标题排序
- [ ] 选择范围（如"第 5-20 个"）
- [ ] 播放列表嵌套支持（频道/合集）

---

## 🐛 已知问题

| # | 问题描述 | 优先级 | 状态 |
|---|---------|--------|
| 1 | `output_path` 偶尔为 null（某些网站 `--print after_move:filepath` 不输出） | 中 | 已知，fallback 到下载目录 |
| 2 | 长文件名在 DownloadItem 中截断后 tooltip 不完整 | 低 | 待优化 |
| 3 | Windows 中文路径含特殊字符时可能编码异常 | 中 | 已设置 PYTHONIOENCODING |
| 4 | 大量任务同时排队时 UI 可能卡顿 | 中 | 待虚拟列表优化 |
| 5 | Linux 剪贴板监听需要安装 `xclip` 或 `xsel` | 低 | 文档说明 |
| 6 | macOS 托盘图标需要实际 icon 文件 | 中 | 需添加图标资源 |
| 7 | `video_info.rs` 中的命令未注册到 invoke_handler | 低 | 预留文件，暂不使用 |
| 8 | 多个 yt-dlp 进程同时运行时 cookie 文件可能冲突 | 低 | 待评估 |
| 9 | `downloaded_size` 字段虽存在但 yt-dlp 输出不包含此信息 | 中 | yt-dlp 限制 |

---

## 📁 必需文件（需手动添加）

| 文件 | 说明 | 状态 |
|------|------|------|
| `src-tauri/binaries/yt-dlp.exe` | yt-dlp 可执行文件 | ❌ 需下载 |
| `src-tauri/binaries/ffmpeg.exe` | ffmpeg 可执行文件（~80MB） | ❌ 需下载 |
| `src-tauri/icons/icon.ico` | Windows 应用图标 | ❌ 需设计 |
| `src-tauri/icons/icon.png` | 通用图标（含托盘图标） | ❌ 需设计 |

---

## 📈 版本历史

### v0.1.0 ✅ 已发布
- 核心下载功能
- 基础 UI（下载页/队列页/设置页）
- 设置持久化
- 错误处理系统
- Cookie 管理

### v0.2.0 ✅ 当前版本
- 播放列表完整支持（解析/选择/批量下载）
- 下载队列并发控制
- 桌面通知（完成/失败）
- 系统托盘（最小化/恢复/退出）
- 剪贴板 URL 自动检测
- yt-dlp 一键更新
- 数据导出/导入（JSON/CSV/设置/URL 列表）
- 国际化基础（中/英）
- 速度限制/缩略图下载/元数据下载
- 音频质量选择

### v0.3.0（计划）
- 主题手动切换（亮/暗/自动）✅ 已完成
- 国际化全覆盖
- 暂停/恢复下载 ✅ 已完成
- 虚拟列表性能优化
- 批量 URL 粘贴 ✅ 已完成
- Cookie 诊断 ✅ 已完成
- 快捷键支持

### v1.0.0（目标）
- 全功能稳定版
- 多平台打包发布（Windows/macOS/Linux）
- 完整 README 文档
- 应用自更新
- 安全增强
