# 跨 TUI / GUI 的 UI 架构抽象指南

## 1. 这份文档回答什么问题

你关心的核心不是“怎么画一个 TUI”或“怎么画一个 GUI”，而是：

> 如果同一套产品以后要同时有 TUI 和 GUI，UI 的公共部分应该怎么抽象，才能让平台相关代码尽可能少？

这里先给出结论。

## 1.1 先给结论

1. **平台无关层不应该直接“render native widget”，而应该产出一个纯数据的 View Tree / Control Tree。**
2. **平台无关层要负责：状态、值编辑语义、校验、格式化、解析、焦点、选择、过滤、命令、状态机。**
3. **平台相关层只负责：把 native 事件翻译成意图，把 View Tree 渲染成 native 控件。**
4. **不要让共享层知道 slider、dropdown、hover、mouse drag 这些平台表现细节；共享层只知道 scalar、choice、reference picker、color、selection list 这类语义控件。**
5. **“tree + render 接口”这个想法方向是对的，但需要修正：**
   - 对的是“tree”
   - 不够对的是“节点自己 render()”
   - 更好的做法是：`State -> ViewTree`，然后 `Renderer.present(ViewTree)`

换句话说：

> **共享层应该输出“要显示什么、可怎么编辑、当前状态是什么”；平台层决定“长什么样、怎么接本地事件”。**

---

## 2. 最重要的边界：什么叫平台无关，什么叫平台相关

## 2.1 平台无关层应该管什么

平台无关层应该管：

- 业务数据
- UI 的交互语义
- 值类型
- 控件抽象
- 校验规则
- 格式化和解析
- 焦点和选择状态
- modal / picker / 输入模式这些状态机
- 用户意图如何修改状态
- 哪些行为会触发 side effect

例如：

- `selection_mix` 是 `0..1` 的标量
- `RuleKind` 是单选枚举
- `SourceRef` 是引用选择器
- `FixedColor` 可以用 hex 输入
- `Enter` 在 source 字段上应该打开 picker
- picker 里输入文本会过滤候选项

这些都应该是共享逻辑。

## 2.2 平台相关层应该管什么

平台相关层应该只管：

- 从本地事件拿数据
- 把本地事件翻译成共享层的 `Intent`
- 把共享层产出的 View Tree 映射成 native 控件
- 管理本地渲染生命周期
- 处理平台特有能力

例如：

- TUI 的按键事件来自 `crossterm`
- GUI 的点击事件来自某个原生 toolkit
- TUI 的 scalar 画成文本条 + 数字输入
- GUI 的 scalar 画成 slider + numeric input

这些应该放平台层。

---

## 3. 为什么“tree”是对的，但“节点自己 render()”通常不够好

你说：

> UI 平台无关这一层的一个模型就是一个 tree，暴露一个 render 接口？

这里有一半非常对，一半需要修正。

## 3.1 对的部分：UI 的共享表达通常确实适合 tree

因为 UI 本身就是分层结构：

- 页面
- 面板
- 列表
- 表单
- 字段
- 对话框
- 预览区

而且焦点、可见性、启用状态、子节点关系，天然都适合 tree。

所以：

> **共享 UI 模型通常应该是 tree。**

这点是对的。

## 3.2 需要修正的部分：共享节点最好不要自己 `render()`

如果你让共享层里的节点定义类似：

```text
trait Widget {
  fn render(&self, ctx: &mut PlatformContext);
}
```

那很容易发生这些问题：

1. 共享层开始知道平台上下文是什么
2. 共享层开始知道具体平台控件是什么
3. 共享层开始混入布局、绘制、尺寸、hover、mouse、theme API
4. 最后“共享层”就不共享了

这会把本来应该留在 renderer 里的东西拉回共享层。

## 3.3 更推荐的做法

共享层只产出纯数据：

```text
State -> ViewTree
```

然后平台层实现：

```text
Renderer.present(&ViewTree)
```

也就是说：

- **ViewTree 是描述**
- **Renderer 是实现**

不是：

- **节点自己直接调用平台绘制**

## 3.4 更精确一点说

你可以让平台无关层暴露：

```text
fn build_view(state: &UiState, domain: &DomainState) -> ViewNode
```

而不是：

```text
fn render(&self, native_ctx: &mut XxxPlatformContext)
```

前者是纯数据构造。
后者会把平台依赖渗进来。

---

## 4. 一个比较稳的跨平台 UI 公共架构

我推荐你把整个 UI 分成 6 层。

## 4.1 Layer 1: Domain Model

这是业务本体。

例如你的 ThemePrism 里：

- `ThemeParams`
- `RuleSet`
- `ResolvedTheme`
- `Palette`
- `Exporter`

这层完全不关心 UI。

---

## 4.2 Layer 2: UI State / Interaction State

这是“用户此刻在怎么编辑”的状态。

例如：

- 当前选中的 token
- 当前 focus 在哪一列
- 当前 inspector 选中了哪个字段
- 当前是不是在输入模式
- 当前 input buffer 是什么
- picker 是否打开
- picker 当前 filter 是什么
- picker 当前选中项是什么
- 当前错误消息 / 状态消息是什么

这层仍然应该是平台无关的。

因为：

- “当前选中了哪项”
- “当前输入 buffer 是什么”
- “当前 picker 是否打开”

这些不是平台细节，而是产品交互状态。

---

## 4.3 Layer 3: Intent / Action / Update

这一层定义：

- 用户做了什么抽象动作
- 系统如何更新状态

例如：

- `MoveSelection(Delta)`
- `AdjustScalar(ControlId, Delta)`
- `BeginTextInput(ControlId)`
- `CommitTextInput`
- `CancelTextInput`
- `OpenSourcePicker`
- `FilterSourcePicker(StringDelta)`
- `ApplySourceSelection`
- `SaveProject`
- `ExportTheme`

这一层的核心是：

```text
fn update(state, intent) -> (new_state, effects)
```

它类似 reducer / state machine。

### 为什么这层必须共享

因为如果这层分散在 TUI 和 GUI 各自实现，你会得到：

- TUI 一套交互语义
- GUI 一套交互语义
- bug 修两遍
- 边界行为不一致

例如：

- picker 过滤规则不一致
- 输入校验不一致
- 步进逻辑不一致
- 空输入怎么处理不一致

这些都应该避免。

---

## 4.4 Layer 4: Control Spec / ViewModel

这是最关键的一层。

它回答：

> 当前有哪些“可编辑控件”，它们分别是什么类型，当前值是什么，允许什么交互？

例如：

```text
ScalarControlSpec {
  id
  label
  value
  min/max
  step
  formatter
  parser
  enabled
  visible
}
```

或：

```text
ReferencePickerSpec {
  id
  label
  selected
  groups
  filter_text
  options
  filterable = true
}
```

### 这层是共享层和 renderer 的接口核心

因为平台层不该直接读业务对象然后自己决定怎么渲染。
它应该消费已经整理好的 ViewModel / ControlSpec。

---

## 4.5 Layer 5: View Tree

这是共享层产出的最终 UI 树。

示意：

```text
Window
  Row
    Panel(TokenList)
    Panel(PreviewPane)
    Panel(InspectorForm)
  StatusBar
  Modal(SourcePicker)
```

或者更细一点：

```text
enum ViewNode {
  Row { children }
  Column { children }
  Panel { title, child }
  SelectionList { ... }
  ScalarField { ... }
  ChoiceField { ... }
  ReferenceField { ... }
  ColorField { ... }
  PreviewPane { ... }
  StatusBar { ... }
  Modal { child }
}
```

### 为什么还要单独有 View Tree

因为 `ControlSpec` 更像“控件定义”，而 `ViewTree` 更像“页面结构”。

你可以理解为：

- `ControlSpec` 关注字段本身
- `ViewTree` 关注布局和组合

---

## 4.6 Layer 6: Platform Adapter / Renderer

这是平台专属实现：

- `TuiRenderer`
- `MacRenderer`
- `WindowsRenderer`
- `GtkRenderer`
- `WebRenderer`

它们做两件事：

1. `NativeEvent -> Intent`
2. `ViewTree -> Native UI`

这层是唯一应该碰平台 API 的地方。

---

## 5. 推荐的数据流

整个系统推荐走单向数据流：

```text
Native Event
  -> Platform Event Adapter
  -> Intent
  -> Shared Update / Reducer
  -> New UI State + Effects
  -> Shared View Builder
  -> ViewTree
  -> Platform Renderer
  -> Native UI
```

这个结构的好处是：

- TUI 和 GUI 共用同一套逻辑
- 渲染可以换
- 事件源可以换
- 业务状态更新路径只有一条

---

## 6. 平台无关层，具体应该放哪些东西

这一节最重要，因为它直接决定你会不会把东西放错层。

## 6.1 应该放在平台无关层的内容

### 1. 值类型定义

例如：

- scalar
- enum
- bool
- set
- text
- color
- reference

### 2. 控件语义

例如：

- 这是一个 `ScalarControl`
- 这是一个 `ReferencePicker`
- 这是一个 `ColorControl`

### 3. 值域规则

例如：

- `selection_mix` 范围 `0..1`
- `hue` 范围 `0..360`
- `FixedColor` 必须是 hex

### 4. formatter / parser

例如：

- 百分比显示成 `35%`
- `0.35` 和 `35%` 都能解析
- hex 输入自动 uppercase

### 5. 校验

例如：

- 输入为空时是否允许
- 输入越界如何处理
- 解析失败如何给出错误

### 6. 步进逻辑

例如：

- 左右一次加 `0.02`
- hue 一次加 `5`

### 7. 焦点模型

例如：

- 当前 focus 在 token list / params / inspector
- Tab 如何移动
- picker 打开时焦点如何捕获

### 8. 模态状态机

例如：

- 当前是不是在输入模式
- 当前是不是在 source picker
- Enter 是 apply 还是 open picker

### 9. 过滤与选择逻辑

例如：

- picker filter 怎么匹配
- 如何维护当前选中项

### 10. side effects 的抽象描述

例如：

- `SaveProject(path)`
- `LoadProject(path)`
- `ExportTheme(target, path)`

注意：

> 是“效果描述”可以共享，不是“怎么调用平台 API”。

---

## 6.2 应该放在平台相关层的内容

### 1. native event 细节

例如：

- `KeyCode::Char('q')`
- 鼠标左键点击
- pointer drag
- window resize

共享层不应该理解这些原始事件。

共享层只应该看到更高层的 `Intent`。

### 2. 实际控件长相

例如：

- GUI 是 slider 还是 spinbox
- TUI 是进度条还是纯文本行

### 3. hover / pointer capture / drag 手势

这些属于平台能力，不该成为共享核心依赖。

### 4. 像素、终端 cell、font metric

例如：

- GUI 的 12px padding
- TUI 的宽度按 cell 算

这些不能放进共享层作为硬编码布局逻辑。

### 5. 平台特有的 dialog / file picker / clipboard API

这些应该通过 adapter 或 service 注入。

---

## 7. 最容易放错层的东西有哪些

这里列一些常见错误。

## 7.1 错误：把原始按键逻辑塞进共享业务

例如在共享层写：

```text
if key == LeftArrow { value -= step }
```

问题：

- GUI 不一定有 LeftArrow 这个交互
- 鼠标拖 slider 时怎么办
- 手柄输入怎么办

更好的方式：

```text
Intent::AdjustScalar(control_id, -1)
```

然后平台层决定 LeftArrow、mouse drag、滚轮，各自怎么翻译成这个 intent。

---

## 7.2 错误：把 slider / dropdown 当作共享抽象

问题在于这些是 GUI 视角的控件名称，不是稳定语义。

共享层应该抽象成：

- `ScalarControl`
- `ChoiceControl`
- `ReferencePicker`

而不是：

- `SliderControl`
- `DropdownControl`

因为 TUI 根本没有等价的 slider / dropdown 外形。

---

## 7.3 错误：共享层直接持有平台对象

例如：

- GUI widget handle
- terminal frame handle
- native callback closure

这会把共享层锁死在某个平台上。

---

## 7.4 错误：把所有 transient UI 状态都丢进平台层

比如：

- 当前输入 buffer
- picker filter
- 当前选中项
- 表单校验错误

这些其实不该是平台层私有状态。
不然 TUI 和 GUI 会有不同的行为。

---

## 7.5 错误：把纯显示细节放进共享层过深

例如：

- 精确的像素 padding
- 终端颜色搭配
- hover 阴影

这些是 renderer 责任，不应该污染共享状态模型。

---

## 8. 你真正需要的公共抽象，不只是 tree，还要有这几块

如果你只保留一个 tree，很多逻辑还是无处安放。

所以更准确地说，公共架构应该至少包含：

## 8.1 Intent

用户意图。

```text
enum Intent {
  MoveFocus(...)
  MoveSelection(...)
  AdjustScalar(...)
  BeginTextInput(...)
  UpdateTextInput(...)
  CommitTextInput
  OpenPicker(...)
  FilterPicker(...)
  SelectPickerNext
  ApplyPickerSelection
  Save
  Load
  Export
}
```

## 8.2 UiState

交互状态。

```text
struct UiState {
  focus
  selected_token
  selected_param
  inspector_field
  text_input
  picker_state
  status_message
}
```

## 8.3 Control Spec

控件编辑语义。

```text
enum ControlSpec {
  Scalar(...)
  Choice(...)
  Boolean(...)
  Text(...)
  ReferencePicker(...)
  Color(...)
}
```

## 8.4 ViewTree

界面结构。

```text
enum ViewNode {
  Container(...)
  Panel(...)
  Field(...)
  List(...)
  Preview(...)
  Status(...)
  Modal(...)
}
```

## 8.5 Effects

需要外部环境执行的事情。

```text
enum Effect {
  SaveProject(path)
  LoadProject(path)
  ExportTheme(path)
  RequestOpenFileDialog
  RequestSaveFileDialog
}
```

## 8.6 Renderer

平台实现。

```text
trait Renderer {
  fn present(&mut self, tree: &ViewNode);
}
```

注意：

> `render()` 应该在 renderer 上，不应该在共享节点上。

---

## 9. 一个更具体的推荐架构

我建议你的共享 UI 架构大概长这样。

## 9.1 DomainState

```text
struct DomainState {
  params: ThemeParams
  rules: RuleSet
  resolved: ResolvedTheme
}
```

## 9.2 UiState

```text
struct UiState {
  focus
  selected_token
  selected_param
  inspector_field
  text_input: Option<TextInputState>
  picker: Option<PickerState>
  status: Option<StatusMessage>
}
```

## 9.3 AppState

```text
struct AppState {
  domain: DomainState
  ui: UiState
}
```

## 9.4 Update

```text
fn update(state: &mut AppState, intent: Intent) -> Vec<Effect>
```

## 9.5 Build View

```text
fn build_view(state: &AppState) -> ViewNode
```

## 9.6 Platform Runtime

```text
loop {
  let native_event = platform.next_event();
  let intents = adapter.map_event(native_event);
  for intent in intents {
    let effects = update(&mut state, intent);
    runtime.run_effects(effects);
  }
  let tree = build_view(&state);
  renderer.present(&tree);
}
```

这就是一套很完整的跨平台公共骨架。

---

## 10. 对你的 ThemePrism 来说，控件层应该怎么抽象

你现在这个项目里，最值得抽象的控件有这些：

## 10.1 `ScalarControl`

用于：

- hue
- lightness
- saturation
- contrast
- selection mix
- adjust amount

建议字段：

```text
id
label
value
min
max
step
display_format
parse_strategy
enabled
```

---

## 10.2 `ChoiceControl`

用于：

- rule kind
- adjust op

建议字段：

```text
id
label
options
selected
style_hint
```

---

## 10.3 `ReferencePickerControl`

用于：

- Alias.source
- Mix.a
- Mix.b
- Adjust.source

建议字段：

```text
id
label
selected
groups
filter_text
options
filterable
```

这是你这个程序里最关键的高级控件之一。

---

## 10.4 `ColorControl`

用于：

- Fixed color

建议字段：

```text
id
label
value: Color
text_repr: "#C586C0"
supports_hex_input: true
supports_channel_edit: true/false
supports_palette_pick: true/false
```

---

## 10.5 `SelectionListControl`

用于：

- token list
- 未来 preset list

建议字段：

```text
id
items
selected
groups
filterable
```

---

## 10.6 `ActionControl`

用于：

- save
- load
- export
- reset

建议字段：

```text
id
label
enabled
```

---

## 11. 事件应该怎么抽象，才能跨 TUI / GUI

关键是：

> 共享层不吃 native event，只吃 semantic intent。

## 11.1 平台事件例子

TUI:

- `KeyCode::Left`
- `KeyCode::Char('e')`
- `KeyCode::Enter`

GUI:

- Slider dragged
- Button clicked
- Text input changed
- Dropdown item selected

这些都不应该直接进入共享业务层。

## 11.2 共享意图例子

你真正应该进入共享 update 的，是：

- `MoveSelectionNext`
- `MoveSelectionPrev`
- `AdjustScalar(control_id, +1)`
- `OpenReferencePicker(control_id)`
- `FilterReferencePicker("accent")`
- `ApplyReferenceSelection(source_ref)`
- `SetTextInput(control_id, value)`
- `CommitTextInput(control_id)`
- `TriggerAction(SaveProject)`

### 这个抽象的好处

这样一来：

- TUI 的 `LeftArrow`
- GUI 的 slider drag
- GUI 的 mouse wheel

都可以统一翻译成：

```text
AdjustScalar(...)
```

共享逻辑完全不需要知道原始输入是什么。

---

## 12. 渲染应该怎么抽象，才能跨 TUI / GUI

## 12.1 不要让共享层决定具体 widget

共享层只决定：

- 控件语义
- 当前值
- 是否可编辑
- 当前是不是 focused / selected / open

平台层决定：

- GUI 用 slider 还是 spinbox
- TUI 用进度条还是纯文本输入

## 12.2 给 renderer 的信息应该足够，但不能太底层

好的 `ViewTree` 应该携带：

- 结构层级
- 节点类型
- label
- value
- 状态
- capabilities
- 可见性
- enabled

但不应该携带：

- 像素阴影
- terminal escape sequence
- hover painting instruction
- toolkit object handle

## 12.3 renderer 最好消费的是“描述”，不是“命令”

不建议共享层输出这种东西：

```text
draw_rect(...)
draw_text(...)
move_cursor(...)
```

因为这已经太接近某种具体绘制 API 了，TUI 和 GUI 很难统一。

更好的方式是输出：

```text
Panel(title="Inspector")
ScalarField(label="Contrast", value="85%")
Modal(title="Source Picker")
```

然后由 renderer 决定怎么画。

---

## 13. 对于 TUI 做不到 GUI 等价控件的情况，正确策略是什么

这是你最关心的另一点。

答案是：

> **不要追求控件外形等价，要追求编辑能力等价。**

## 13.1 例子：Slider

GUI:

- 可拖动 slider

TUI:

- 左右键步进
- 文本输入
- 文本进度条

### 共享抽象

- `ScalarControl`

### 等价点

- 都能精确编辑标量
- 都有步进能力
- 都能表达当前值

---

## 13.2 例子：Color Wheel / 2D Color Panel

GUI:

- 色轮
- 2D 饱和度/亮度平面

TUI:

- hex input
- H/S/L 三个 scalar
- 预设色 picker

### 共享抽象

- `ColorControl`

### 等价点

- 最终编辑的是同一个颜色值

---

## 13.3 例子：Dropdown / Searchable Select

GUI:

- 下拉菜单
- 搜索选择器

TUI:

- modal picker
- filterable command palette

### 共享抽象

- `ChoiceControl` 或 `ReferencePicker`

---

## 13.4 例子：Hover Tooltip

GUI:

- 鼠标悬停提示

TUI:

- 状态栏帮助
- focused field 的内联说明

### 共享抽象

- `HelpText` / `ContextHint`

也就是说，“帮助信息”是共享的，“hover”不是共享的。

---

## 14. 共享层里最应该抽象成什么：树，还是 schema？

最实际的答案是：

> **两者都要，但职责不同。**

## 14.1 Tree 适合表达结构

适合表达：

- 页面结构
- panel 嵌套
- modal
- 列表
- preview 布局

## 14.2 Schema / Spec 适合表达控件语义

适合表达：

- 这是 scalar
- 范围是多少
- step 是多少
- parser 是什么
- 可不可以过滤
- 当前 options 是什么

所以更稳的做法不是“只有一个 tree”，而是：

- **Tree 表达结构**
- **Spec 表达控件语义**

ViewTree 的叶子节点可以挂 `ControlSpec`。

例如：

```text
ViewNode::Field(ControlSpec::Scalar(...))
ViewNode::Field(ControlSpec::ReferencePicker(...))
```

---

## 15. 一个很推荐的公共接口形状

下面给一个你后面可以真的落代码的形状。

## 15.1 共享层暴露这些接口

```text
trait AppLogic {
  fn update(&mut self, intent: Intent) -> Vec<Effect>;
  fn build_view(&self) -> ViewNode;
}
```

或者拆得更清楚：

```text
fn update(state: &mut AppState, intent: Intent) -> Vec<Effect>;
fn build_view(state: &AppState) -> ViewNode;
```

## 15.2 平台层暴露这些接口

```text
trait EventAdapter<NativeEvent> {
  fn map_event(&mut self, event: NativeEvent) -> Vec<Intent>;
}

trait Renderer {
  fn present(&mut self, tree: &ViewNode);
}

trait EffectRunner {
  fn run(&mut self, effect: Effect);
}
```

### 这套分离很关键

- `EventAdapter` 负责输入翻译
- `Renderer` 负责显示
- `EffectRunner` 负责外部副作用

这样共享层不会碰平台 API。

---

## 16. 对你当前项目的直接架构建议

如果你真的准备让这个项目以后同时有 TUI 和 GUI，我建议后面逐渐往这个目录方向调整：

```text
src/
  domain/
    params.rs
    palette.rs
    rules.rs
    evaluator.rs

  app/
    state.rs
    intent.rs
    update.rs
    effects.rs
    view.rs
    controls.rs

  platform/
    tui/
      renderer.rs
      event_adapter.rs
      runtime.rs
    gui/
      renderer.rs
      event_adapter.rs
      runtime.rs
```

### `domain/`

纯业务。

### `app/state.rs`

UI 交互状态。

### `app/intent.rs`

共享 intent。

### `app/update.rs`

状态机 / reducer。

### `app/view.rs`

构建 ViewTree。

### `app/controls.rs`

定义 `ScalarControl`, `ChoiceControl`, `ReferencePicker`, `ColorControl` 这些 spec。

### `platform/tui/*`

只做 TUI 的事件翻译和绘制。

### `platform/gui/*`

只做 GUI 的事件翻译和绘制。

---

## 17. 一句话总结：公有 UI 架构到底应该是什么

如果要把这套架构压成一句话，我会这样说：

> **共享层维护 State、Intent、Update、ControlSpec、ViewTree、Effect；平台层只维护 NativeEventAdapter、Renderer、EffectRunner。**

再压缩一点：

> **共享层决定“交互语义和状态变化”，平台层决定“事件来源和可视化呈现”。**

---

## 18. 最终建议

如果你之后真的要做 native GUI，我建议你把“公共抽象”的核心只放这几类：

1. `AppState`
2. `Intent`
3. `Effect`
4. `ControlSpec`
5. `ViewTree`
6. `update()`
7. `build_view()`

不要把下面这些塞进共享核心：

- 原始按键
- 鼠标拖拽细节
- hover
- toolkit widget handle
- terminal frame API
- 像素绘制命令

最后再回答你那个问题：

> “平台无关这一层的一个模型就是一个 tree，暴露一个 render 接口？”

我的答案是：

- **Tree：对**
- **render 接口：要放在 renderer 上，不要放在共享节点上**

更准确的版本应该是：

```text
platform-agnostic:
  State
  Intent
  Update
  ControlSpec
  ViewTree

platform-specific:
  NativeEvent -> Intent
  ViewTree -> Native UI
```

这才是一个比较稳定、长期不会把边界搞乱的跨 TUI / GUI 公共架构。
