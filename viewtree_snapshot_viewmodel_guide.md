# ViewTree / Snapshot / ViewModel 边界说明

## 1. 这份文档回答什么问题

你现在已经接受一件事：

- 业务核心尽量长期留在 Rust
- TUI / Rust GUI 也要分层，最大化复用
- 将来如果体验不理想，可能会切到 “AppKit / SwiftUI shell + Rust core”

这时就会出现一个关键问题：

> 之前我们说共享层输出 `ViewTree`。  
> 但如果以后 Rust 只当 core，外面是 AppKit/SwiftUI，那么还应该输出 `ViewTree` 吗？  
> `Snapshot`、`ViewModel` 和 `ViewTree` 到底有什么区别？

这份文档就是专门把这个边界讲清楚。

---

## 2. 先给最短结论

### `ViewTree`

给 **同语言、同进程、同渲染体系里的 renderer** 用的 UI 描述树。

它回答的是：

> “这一帧 / 这一刻，UI 要怎么组织和显示？”

它更接近：

- 布局树
- 面板树
- 控件树
- overlay / modal 树

适合：

- Rust TUI renderer
- Rust 内部的 GUI renderer

---

### `ViewModel`

给 **某个界面、某个面板、某个视图** 用的“展示模型”。

它回答的是：

> “这个具体视图需要哪些数据和交互状态？”

它更接近：

- `InspectorViewModel`
- `TokenListViewModel`
- `PalettePanelViewModel`

适合：

- 原生 GUI 的控制器 / view / view controller
- 某个面板级别的数据绑定

---

### `Snapshot`

给 **跨边界读取** 用的、在某个时间点冻结下来的 UI / 应用状态投影。

它回答的是：

> “外部 shell 现在应当看到的完整状态是什么？”

它更接近：

- 序列化友好的状态快照
- FFI 友好的读取模型
- shell 拉取 UI 数据的统一结果

适合：

- AppKit / SwiftUI shell 调 Rust core
- IPC / FFI / 插件宿主
- 调试、测试、录制

---

## 3. 三者最核心的区别

最重要的一句是：

> **`ViewTree` 主要是为了“渲染”。`Snapshot` 主要是为了“跨边界传输”。`ViewModel` 主要是为了“给某个具体视图消费”。**

它们不是互斥概念，而是不同层的产物。

---

## 4. 按维度比较

## 4.1 关注点不同

### `ViewTree`

关注：

- 节点结构
- 布局层级
- 子节点关系
- 可见性
- 样式描述

例如：

- 左中右三栏
- 面板标题
- 表单字段顺序
- 弹窗 overlay

---

### `ViewModel`

关注：

- 某个视图所需的数据
- 某个视图内部的选择状态
- 某个视图的命令入口
- 对原生控件友好的字段

例如：

- Inspector 需要：
  - 当前 token 名称
  - 当前颜色
  - 当前 rule 类型
  - 当前可编辑字段列表
  - 当前输入提示

---

### `Snapshot`

关注：

- 一个时间点的完整、稳定、可导出的外部视图
- 跨边界读取时的兼容性
- 序列化 / FFI 友好

例如：

- 整个应用当前有几个 pane
- 当前选中了哪个 token
- 当前 palette 是什么
- Inspector 当前字段是什么
- 当前是否打开 source picker

---

## 4.2 消费者不同

### `ViewTree` 的消费者

- Rust TUI renderer
- Rust GUI renderer

它通常由同一个 crate 里的 renderer 直接消费。

---

### `ViewModel` 的消费者

- 某个 native view controller
- 某个具体 widget subtree
- SwiftUI/AppKit 的某个面板

它通常是“局部消费”。

---

### `Snapshot` 的消费者

- 外部 shell
- FFI bridge
- 调试工具
- 测试 harness

它通常是“边界消费”。

---

## 4.3 稳定性要求不同

### `ViewTree`

可以较快演进。

因为它往往只在 Rust 内部使用，改结构的成本相对低。

---

### `ViewModel`

稳定性中等。

因为它面对的是某个特定视图实现。这个视图一旦写出来，就会依赖它的字段形状。

---

### `Snapshot`

稳定性要求最高。

因为一旦跨语言、跨模块、跨进程，它就成了边界协议的一部分。

这时你要非常注意：

- 字段命名
- 版本演进
- identity
- 序列化格式

---

## 4.4 是否适合跨 FFI

### `ViewTree`

通常 **不适合直接跨 FFI**。

原因：

- 节点种类多
- 递归结构复杂
- 带大量 Rust enum / String / Vec / style 细节
- 一旦跨语言，维护成本很高

---

### `ViewModel`

有时适合，但前提是你把它做成 FFI 友好的普通结构。

例如：

- `InspectorViewModel`
- `PaletteViewModel`

这些可以拆成较稳定的扁平结构。

---

### `Snapshot`

最适合跨 FFI。

因为它本来就是为“边界读取”设计的。

---

## 5. 一个容易混淆但非常重要的点

很多人会把 `Snapshot` 和 `ViewModel` 混成一个词。

这是因为它们经常长得很像，但重点不一样。

### `Snapshot` 强调的是“时刻”

它是：

> 在某个时间点，把当前 UI / 应用状态冻结出来的一份对外结果。

它强调：

- 当前时刻
- 一次读取
- 边界传输

---

### `ViewModel` 强调的是“用途”

它是：

> 某个 view 为了渲染和交互而需要的数据模型。

它强调：

- 为谁服务
- 服务哪块界面
- 需要什么字段

---

所以一句话区分：

> **`Snapshot` 是“何时、如何拿出去”。`ViewModel` 是“给谁用、长什么样”。**

---

## 6. 在你当前架构里，它们分别放哪

你当前项目现在更接近这个结构：

```text
State
  -> build_view()
  -> ViewTree
  -> TUI Renderer
```

这是完全合理的。

因为：

- TUI renderer 在 Rust 内部
- renderer 和 shared layer 同语言、同进程
- `ViewTree` 的复杂结构不会穿过 FFI

所以：

> **对当前 Rust TUI 版本，`ViewTree` 是对的。**

---

## 7. 如果以后变成 “Rust 只当 core”，哪里会变

以后如果你切到：

```text
AppKit / SwiftUI shell
  -> Rust core
```

那最关键的变化是：

> **native shell 不应该直接吃 Rust 内部的 `ViewTree`。**

更推荐的是：

```text
native event
  -> bridge
  -> Rust core dispatch(intent)
  -> state update
  -> snapshot()
  -> native shell rebuild / patch UI
```

也就是说：

- `Intent`
- `Update`
- `Effect`
- `Domain`

这些还留在 Rust

但 `ViewTree -> Renderer` 这部分会变成：

- Rust 输出 `Snapshot`
- AppKit/SwiftUI 自己渲染 native UI

---

## 8. 所以之前的架构会不会被推翻

不会。

真正要保住的是这条主干：

```text
Native Event
  -> Intent
  -> Update(State)
  -> Effects
```

和这些共享核心：

- domain model
- control semantics
- ui state
- parsing / formatting
- validation
- effect descriptions

这些在两种架构下都成立。

真正可能替换的是最后一段：

### 当前

```text
State -> ViewTree -> Rust Renderer
```

### 将来

```text
State -> Snapshot/ViewModel -> Native Shell
```

所以不是推翻，而是把“最后一段 UI 输出形式”换掉。

---

## 9. 当前项目最合理的理解方式

你现在可以把系统理解成两层输出：

## 9.1 第一层：交互核心输出

这是不会轻易变的：

- `Intent`
- `Update`
- `Effect`
- `AppState`

这是系统的真正 core。

---

## 9.2 第二层：呈现输出

这里有两种可能：

### 呈现输出 A：`ViewTree`

给 Rust 自己的平台层使用。

适合：

- TUI
- 未来如果你坚持 Rust GUI renderer

---

### 呈现输出 B：`Snapshot / ViewModel`

给 native shell 使用。

适合：

- AppKit
- SwiftUI
- 未来如果做独立 GUI app

---

## 10. 一个推荐的具体分层

我建议你以后把共享层想成：

```text
Domain
App State
Intent
Update
Effect
Output Builders
```

这里的 `Output Builders` 可以同时有两套：

```text
build_view_tree(state) -> ViewTree
build_snapshot(state) -> AppSnapshot
```

这样：

- TUI 用 `ViewTree`
- native shell 用 `AppSnapshot`

两边都复用同一份 core。

---

## 11. `ViewTree` 和 `Snapshot` 可以同时存在吗

可以，而且这通常是最合理的。

这两个不是互斥设计。

更好的理解是：

- `ViewTree` 是 **内部渲染描述**
- `Snapshot` 是 **外部边界描述**

你完全可以同时保留：

```text
crate::app::view
crate::app::snapshot
```

这反而比“强行只保留一种输出”更健康。

---

## 12. 你的主题生成器里，`Snapshot` 应该长什么样

如果以后给 AppKit shell 用，我建议它不要做成一个巨大无语义的通用树。

更好的是：

```text
AppSnapshot {
  window: WindowSnapshot,
  theme: ThemeSnapshot,
  token_list: TokenListViewModel,
  params_panel: ParamsPanelViewModel,
  preview_panel: PreviewViewModel,
  palette_panel: PaletteViewModel,
  resolved_tokens_panel: ResolvedTokensViewModel,
  inspector_panel: InspectorViewModel,
  overlay: Option<OverlayViewModel>,
  status_bar: StatusBarViewModel,
}
```

这里你会看到：

- `Snapshot` 是整体
- 里面装的是一组按功能拆开的 `ViewModel`

这通常是最稳的。

---

## 13. 为什么不建议把 `ViewTree` 直接穿到 AppKit

原因主要有 6 个。

### 13.1 树太“渲染导向”

它更像 renderer 的输入，而不是 shell 的业务接口。

AppKit 不一定关心：

- `Split`
- `Panel`
- `SwatchList`
- `Overlay`

它可能更关心：

- 这个 sidebar 的 items 是什么
- 这个 inspector 的字段是什么
- 这个 picker 的候选项是什么

---

### 13.2 风格信息会污染边界

`ViewTree` 里经常带：

- 布局方向
- 尺寸约束
- span style
- 排版细节

这些在跨 FFI 时会变得很脆。

---

### 13.3 原生 shell 自己有布局系统

AppKit/SwiftUI 会自己决定：

- sidebar
- split view
- inspector
- toolbar
- sheet

这时你不需要 Rust 告诉它“怎么摆”，更需要 Rust 告诉它“有什么内容、当前状态是什么”。

---

### 13.4 跨语言递归树维护成本高

递归 enum 树一旦跨 FFI，成本会明显提高。

---

### 13.5 identity 会更难做

原生 UI 很关心：

- 哪一项被选中
- 哪个控件应该保留焦点
- 哪个列表项只是更新，不是重建

这些在通用树里可以做，但通常不如显式 view model 清楚。

---

### 13.6 原生 UI 更喜欢语义模型

例如：

- `NSTableView` 喜欢 row data
- `NSOutlineView` 喜欢 tree data with identity
- 表单控件喜欢字段数组

这些都更适合 `ViewModel`。

---

## 14. 一个你可以长期采用的折中方案

你当前完全可以坚持：

```text
Rust core + Rust TUI/GUI platform adapters
```

同时在设计上预留：

```text
Rust core + Snapshot/FFI + Native shell
```

最好的做法不是现在就切架构，而是：

1. 保持当前 `Intent / Update / Effect / AppState`
2. 保持当前 `ViewTree` 给 Rust TUI 用
3. 未来新增 `snapshot.rs`
4. 再未来新增 `ffi.rs`

这样业务层完全不需要重写。

---

## 15. 推荐的未来目录方向

如果以后继续演进，我建议大致长成这样：

```text
src/
  domain/
  app/
    state.rs
    intent.rs
    effect.rs
    update.rs
    controls.rs
    view.rs
    snapshot.rs
  platform/
    tui/
    rust_gui/
    gui_bridge/
  ffi/
```

含义是：

- `view.rs` 给 Rust 内部 renderer
- `snapshot.rs` 给 native shell / FFI
- `ffi/` 定义跨语言边界

---

## 16. 对你这个项目的最终建议

这里给出最明确的结论。

### 现在

继续按当前路线做：

- Rust 写 core
- Rust 里保留 TUI
- Rust 平台层和共享层分离
- 继续用 `ViewTree`

这是对的。

---

### 以后如果接 AppKit / SwiftUI

不要让 native shell 直接吃 `ViewTree`。

应该新增：

- `AppSnapshot`
- 若干 `ViewModel`
- FFI 边界

这也是对的。

---

### 所以两套思路不是冲突，而是层次不同

一句话总结：

> **`ViewTree` 是 Rust 内部多平台渲染的公共输出。`Snapshot/ViewModel` 是 Rust core 对 native shell 的公共输出。**

两者可以并存，而且通常应该并存。

---

## 17. 最后的记忆口诀

如果你想快速记住它们的区别，可以记这三句：

1. **`ViewTree`：给 renderer 看。**
2. **`ViewModel`：给具体 view 看。**
3. **`Snapshot`：给边界外部看。**

再补一句：

4. **`Intent / Update / Effect / State` 才是真正最稳定的 core。**

