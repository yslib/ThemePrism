# Theme Generator Product And Architecture Plan

## Product Goal

这个项目的目标不是一个 demo，而是一个真正可长期维护的主题设计产品。

它要同时满足几件事：

- 让主题设计、色彩关系和导出配置形成完整工作流
- 为 terminal 重度用户提供正式可用的 TUI
- 为设计师和普通用户提供 native GUI
- 让业务逻辑长期只维护一份
- 让单人维护成本尽可能低

产品价值不只来自颜色计算，还来自完整工作流：

- 参数调节和规则编辑
- preview 和反馈
- 工程文件保存与加载
- exporter / template system
- 系统能力接入
- 长期稳定的架构边界

## Guiding Decisions

当前阶段采用下面的路线：

1. Rust 保持为唯一业务核心
2. TUI 保持一等公民，并优先跑通全部业务能力
3. GUI 必须是 native，以保证产品质感和平台契合度
4. 当前 GUI 允许使用轻量 bridge 方式先落地
5. 未来逐步演进到 typed FFI，让 Rust core 成为 native GUI 可复用的库

这意味着：

- 短期不追求最理想的 native shell 结构
- 短期优先保证业务逻辑、工程能力和导出能力完整
- 中期把共享核心边界收紧
- 长期再把 GUI 边界从字符串 / JSON bridge 升级成 typed FFI

## Stable Core Boundary

真正应该长期稳定的层是：

- `Domain`
- `State`
- `Intent`
- `Update`
- `Effect`

围绕这个核心，再提供两类输出：

- `ViewTree`
  - 面向 Rust 内部 renderer
  - 主要服务 TUI

- `Snapshot / ViewModel`
  - 面向 native host 或 FFI 边界
  - 主要服务 native GUI

因此：

- TUI 可以继续吃 `ViewTree`
- GUI 可以继续吃 `Snapshot`
- 两者共享同一个 Rust core

## Platform Strategy

### TUI

TUI 是产品主工作台。

要求：

- 所有核心业务能力先在 TUI 跑通
- TUI 不是测试壳，而是长期存在的正式接口
- 任何复杂业务编辑逻辑都优先在 TUI 落地并验证

### Native GUI

GUI 当前目标不是抢先承担全部业务，而是：

- 以 native 方式表达共享业务状态
- 提供更产品化、更精致的交互外观
- 验证未来 native shell 的可行路径

短期：

- 允许使用 bridge + snapshot

长期：

- 逐步改成 typed FFI
- 最终让 Rust core 成为 native GUI 可调用的库

## Phase Plan

### Phase 1: Tighten Core Boundary

目标：

- 明确什么属于 `core`
- 让平台层通过统一会话 API 驱动业务
- 降低 TUI / GUI 对内部状态细节的直接依赖

要做的事：

- 引入共享 `CoreSession`
- 收口 `state + intent dispatch + effect dispatch + view + snapshot`
- 让平台入口优先依赖 `CoreSession`
- 逐步把副作用执行逻辑从平台层移回共享核心边界

完成标准：

- TUI 和 GUI 都通过同一套核心会话驱动
- 平台层不直接自己拼装业务更新链路

### Phase 2: Complete Product Logic In TUI

目标：

- 让 TUI 成为真正可用的完整产品工作台

优先内容：

- 工程文件模型完善
- 保存 / 加载 / 最近项目策略
- exporter / template system
- 更完整的 inspector
- 更强的 preview
- 错误反馈、状态提示、默认值与重置
- 撤销 / 重做

完成标准：

- 即使没有 GUI，这个产品也已经能独立成立

### Phase 3: Keep Native GUI Thin But Productive

目标：

- GUI 保持 native 表现层
- 不在 GUI 内复制业务系统

要做的事：

- 改善 native 布局和控件体验
- 实时预览
- 文件系统与原生对话框接入
- GUI 只负责：
  - native event -> intent
  - snapshot/viewmodel -> native widgets

完成标准：

- GUI 在产品层面可用
- 但业务规则仍只维护在 Rust core

### Phase 4: Upgrade Bridge To Typed FFI

目标：

- 让 Rust core 真正成为 native GUI 可调用的稳定库

要做的事：

- 定义 `SessionHandle`
- 定义 typed intent dispatch API
- 定义 typed snapshot / viewmodel 输出
- 定义 effect 交换边界
- 逐步减少字符串命令和手写 JSON

完成标准：

- native shell 可以直接调用 Rust core
- 未来可平滑切到 AppKit / SwiftUI shell

## Immediate Execution Order

当前接下来按下面顺序推进：

1. 引入 `CoreSession`，收紧共享核心边界
2. 让 TUI runtime 和 AppKit bridge 改走 `CoreSession`
3. 清理当前 bridge 中过于平台化的业务分发逻辑
4. 开始补 TUI 侧完整业务能力
5. GUI 只继续做表现与系统集成
6. 业务闭环稳定后，再设计 typed FFI

## Non-Goals For The Current Stage

当前阶段不做下面这些事：

- 不立刻把整个 GUI 重写成 SwiftUI shell
- 不急着做 C/S 架构
- 不为了“最优原生架构”牺牲业务推进速度
- 不把平台细节反向污染进共享业务层

## Success Criteria

如果路线正确，后续应该达到这些状态：

- 业务规则始终只在 Rust 写一份
- TUI 始终可独立使用
- GUI 始终是 native 的
- GUI 的平台代码只管表现和事件适配
- 以后切 typed FFI 时，不需要推翻当前业务核心
